"""Shared daemon-thread ThreadPoolExecutor.

Stdlib ``ThreadPoolExecutor`` workers are non-daemon AND are registered in
``concurrent.futures.thread._threads_queues``, whose atexit hook
(``_python_exit``) joins every worker unconditionally — even after
``shutdown(wait=False)``.  A single wedged worker (tool blocked on network
I/O, hung provider daemon, stuck subagent) therefore blocks interpreter
exit forever.  This is the root cause of multi-minute CLI exits on long
sessions: every abandoned concurrent-tool batch leaves workers that the
exit hook insists on joining.

``DaemonThreadPoolExecutor`` spawns daemon workers and skips the
``_threads_queues`` registration, so:

  - ``_python_exit`` never joins them, and
  - the interpreter's non-daemon thread join at shutdown skips them.

Semantics are otherwise identical (initializer/initargs, work queue,
idle-thread reuse).  Use it for any pool whose work is best-effort or
independently interruptible and must never hold the process open:
concurrent tool execution, background memory sync, catalog fan-out,
subagent timeout wrappers.  Do NOT use it for work that must complete
before exit (durable writes) — those belong on foreground threads with
explicit bounded joins.
"""

from __future__ import annotations

import threading
import weakref
from concurrent.futures import ThreadPoolExecutor
from concurrent.futures.thread import _worker

__all__ = ["DaemonThreadPoolExecutor"]


class DaemonThreadPoolExecutor(ThreadPoolExecutor):
    """ThreadPoolExecutor variant whose workers do not block process exit.

    Works across CPython 3.11–3.12 and 3.13+ (including 3.14), where the
    worker contract changed: in 3.11/3.12 ``_worker`` is called as
    ``(executor_ref, work_queue, initializer, initargs)`` and the executor
    stores ``_initializer``/``_initargs`` on the instance; in 3.13+ the
    initializer is folded into a ``_WorkerContext`` returned by
    ``_create_worker_context()`` and ``_worker`` is called as
    ``(executor_ref, ctx, work_queue)`` — ``_initializer``/``_initargs`` no
    longer exist on the instance.  Detect the contract at runtime instead of
    pinning a version so the pool survives interpreter upgrades.
    """

    # Set in __init__ below; True on 3.13+ (no per-instance _initializer).
    _uses_worker_context = False

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        # 3.13+ wraps the initializer in a context object; older versions
        # keep _initializer/_initargs as plain instance attributes.
        self._uses_worker_context = hasattr(self, "_create_worker_context")

    def _adjust_thread_count(self) -> None:
        # Mirrors CPython's implementation with two changes: daemon=True and
        # no _threads_queues registration.  The worker-args shape differs
        # between interpreter versions (see class docstring).
        if self._idle_semaphore.acquire(timeout=0):
            return

        def weakref_cb(_, q=self._work_queue):
            q.put(None)

        num_threads = len(self._threads)
        if num_threads < self._max_workers:
            thread_name = "%s_%d" % (self._thread_name_prefix or self, num_threads)
            if self._uses_worker_context:
                worker_args = (
                    weakref.ref(self, weakref_cb),
                    self._create_worker_context(),
                    self._work_queue,
                )
            else:
                worker_args = (
                    weakref.ref(self, weakref_cb),
                    self._work_queue,
                    self._initializer,
                    self._initargs,
                )
            t = threading.Thread(
                name=thread_name,
                target=_worker,
                args=worker_args,
                daemon=True,
            )
            t.start()
            self._threads.add(t)
