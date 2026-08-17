---
name: React Patterns
description: Modern React 18+ patterns and best practices for building scalable applications
triggers:
  - react patterns
  - react hooks
  - react component design
  - react 18
  - react 19
version: 1.0.0
tags:
  - react
  - frontend
  - hooks
  - components
---

# React Patterns (React 18+)

## Hook Patterns

### useState with Functional Update
```jsx
const [count, setCount] = useState(0);
// Functional update for dependent state
setCount(prev => prev + 1);
```

### useEffect Cleanup
```jsx
useEffect(() => {
  const subscription = subscribe(id, handler);
  return () => subscription.unsubscribe(); // cleanup
}, [id]);
```

### Custom Hook Pattern
```jsx
function useDebounce(value, delay) {
  const [debouncedValue, setDebouncedValue] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);
  return debouncedValue;
}
```

## Component Patterns

### Compound Component
```jsx
function Tabs({ children }) {
  const [activeTab, setActiveTab] = useState(0);
  return (
    <TabsContext.Provider value={{ activeTab, setActiveTab }}>
      {children}
    </TabsContext.Provider>
  );
}

Tabs.Panel = function TabPanel({ index, children }) {
  const { activeTab } = useContext(TabsContext);
  return activeTab === index ? children : null;
};
```

### Render Props
```jsx
function MouseTracker({ render }) {
  const [pos, setPos] = useState({ x: 0, y: 0 });
  useEffect(() => {
    const handler = e => setPos({ x: e.clientX, y: e.clientY });
    window.addEventListener('mousemove', handler);
    return () => window.removeEventListener('mousemove', handler);
  }, []);
  return render(pos);
}
```

### Container/Presentational
```jsx
// Presentational
function UserList({ users, onDelete }) {
  return (
    <ul>
      {users.map(u => (
        <li key={u.id}>
          {u.name}
          <button onClick={() => onDelete(u.id)}>Delete</button>
        </li>
      ))}
    </ul>
  );
}

// Container
function UserListContainer() {
  const [users, setUsers] = useState([]);
  const deleteUser = async (id) => {
    await api.deleteUser(id);
    setUsers(prev => prev.filter(u => u.id !== id));
  };
  return <UserList users={users} onDelete={deleteUser} />;
}
```

## React 18+ Patterns

### Suspense for Data Fetching
```jsx
function App() {
  return (
    <Suspense fallback={<Spinner />}>
      <UserProfile userId={id} />
    </Suspense>
  );
}
```

### useTransition for Non-blocking Updates
```jsx
function SearchResults({ query }) {
  const [isPending, startTransition] = useTransition();
  const [results, setResults] = useState([]);

  startTransition(() => {
    setResults(expensiveSearch(query));
  });

  return isPending ? <Spinner /> : <ResultsList data={results} />;
}
```

### useDeferredValue
```jsx
function SearchBox() {
  const [query, setQuery] = useState('');
  const deferredQuery = useDeferredValue(query);
  // Use deferredQuery for expensive filtering
}
```

### Concurrent Features Pattern
```jsx
function Search() {
  const [input, setInput] = useState('');
  const deferredInput = useDeferredValue(input);
  
  return (
    <div>
      <input value={input} onChange={e => setInput(e.target.value)} />
      <Suspense fallback={<Loading />}>
        <SearchResults query={deferredInput} />
      </Suspense>
    </div>
  );
}
```

## State Management Patterns

### useReducer for Complex State
```jsx
function reducer(state, action) {
  switch (action.type) {
    case 'INCREMENT': return { count: state.count + 1 };
    case 'DECREMENT': return { count: state.count - 1 };
    case 'RESET': return { count: 0 };
    default: return state;
  }
}

function Counter() {
  const [state, dispatch] = useReducer(reducer, { count: 0 });
  return (
    <>
      <span>{state.count}</span>
      <button onClick={() => dispatch({ type: 'INCREMENT' })}>+</button>
    </>
  );
}
```

### Context with Reducer
```jsx
const AppContext = createContext();

function AppProvider({ children }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}
```

## Performance Patterns

### Memoization
```jsx
const MemoizedComponent = React.memo(function MyComponent({ data }) {
  return <div>{data}</div>;
});

// With custom equality check
const MemoizedExpensive = React.memo(Expensive, (prev, next) => 
  prev.id === next.id && prev.filter === next.filter
);
```

### useMemo and useCallback
```jsx
const expensiveValue = useMemo(() => computeExpensive(items), [items]);

const handleClick = useCallback(() => {
  doSomething(id);
}, [id]);
```

### Virtualization for Lists
```jsx
import { FixedSizeList } from 'react-window';

function VirtualList({ items }) {
  return (
    <FixedSizeList height={400} itemCount={items.length} itemSize={50}>
      {({ index, style }) => (
        <div style={style}>{items[index].name}</div>
      )}
    </FixedSizeList>
  );
}
```

## React 19+ Patterns

### Server Components
```jsx
// Server Component (default)
async function UserProfile({ userId }) {
  const user = await db.getUser(userId);
  return <div>{user.name}</div>;
}
```

### use() Hook
```jsx
import { use } from 'react';

function UserProfile({ userPromise }) {
  const user = use(userPromise);
  return <div>{user.name}</div>;
}
```

### Actions
```jsx
function Form() {
  async function submit(formData) {
    'use server';
    await db.create(formData);
  }
  return <form action={submit}>...</form>;
}
```

## Error Handling

### Error Boundaries
```jsx
class ErrorBoundary extends React.Component {
  state = { hasError: false };
  static getDerivedStateFromError() { return { hasError: true }; }
  
  componentDidCatch(error, info) {
    logError(error, info);
  }

  render() {
    if (this.state.hasError) return <FallbackUI />;
    return this.props.children;
  }
}
```

## Best Practices

1. **Colocation**: Keep components, hooks, and styles close to where they're used
2. **Composition over Inheritance**: Build complex UIs from simple pieces
3. **Early Returns**: Return null/fallback early to reduce nesting
4. **Prop Types**: Use TypeScript or PropTypes for type safety
5. **Single Responsibility**: Each component should do one thing well
6. **Custom Hooks**: Extract reusable stateful logic into hooks
7. **Server vs Client**: Prefer Server Components; use 'use client' only when needed
