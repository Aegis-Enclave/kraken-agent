import { Box, Text } from '@hermes/ink'
import { useStore } from '@nanostores/react'

import { $uiState } from '../app/uiStore.js'
import { KRAKEN_BORDER, type Theme } from '../theme.js'

// ── The Raven ASCII art — art-nouveau dragon-raven with spread wings,
//    swirl motifs, and a dark gothic silhouette ─────────────────────

const RAVEN_ART = [
  '  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  ',
  ' ███████████████████████ ',
  ' ███████████████████████ ',
  '█████████████████████████',
  '█████████████████████████',
  '█████████████████████████',
  '████████▀▀▀▀▀▀▀█████████',
  '██████▀       ▀██████████',
  '██████   ▄▄   ▐██████████',
  '██████  ████  ▐██████████',
  '██████   ▀▀   ▐██████████',
  '███████▄     ▄███████████',
  '▐████████████████████████▌',
  ' ▀██████████████████████▀ ',
  '  ███████████████████████ ',
  ' ▄███████████████████████▄',
  '███████████████████████████',
  '███████████████████████████',
  '███████████████████████████',
  '▀█████████████████████████▀',
  '  ▀█████████████████████▀  ',
  '    ▀▀███████████████▀▀    ',
  '       ▀▀▀▀▀▀▀▀▀▀▀▀       '
]

const RAVEN_WIDTH = RAVEN_ART.reduce((m, l) => Math.max(m, l.length), 0)

// ── Decorative divider ─────────────────────────────────────────────

function Divider({ t }: { t: Theme }) {
  return (
    <Text color={t.color.border} dim>
      {'▀'.repeat(20)}
    </Text>
  )
}

function ThinDivider({ t }: { t: Theme }) {
  return (
    <Text color={t.color.muted} dim>
      {'─'.repeat(20)}
    </Text>
  )
}

// ── Art render ─────────────────────────────────────────────────────

function RavenArt({ t }: { t: Theme }) {
  // Gradient: outer wing tips (muted) → wings (border) → body/accent →
  // center chest/head (primary) → tail (muted), creating a luminous
  // silhouette against the dark background.
  const gradient = [3, 2, 2, 1, 0, 0, 1, 1, 0, 0, 1, 2, 2, 3, 2, 1, 0, 0, 1, 2, 3, 3, 2] as const
  const p = [t.color.primary, t.color.accent, t.color.border, t.color.muted]

  return (
    <Box flexDirection="column" height={RAVEN_ART.length} width={RAVEN_WIDTH}>
      {RAVEN_ART.map((line, i) => {
        const c = p[gradient[i]!] ?? t.color.muted
        return (
          <Text key={i} color={c}>
            {line}
          </Text>
        )
      })}
    </Box>
  )
}

// ── Info lines ─────────────────────────────────────────────────────

function InfoLine({ label, value, t }: { label: string; value: string; t: Theme }) {
  return (
    <Text wrap="truncate-end">
      <Text color={t.color.muted} dim>
        {label}{' '}
      </Text>
      <Text color={t.color.text}>{value}</Text>
    </Text>
  )
}

function StatusDot({ active, t }: { active: boolean; t: Theme }) {
  return <Text color={active ? t.color.statusGood : t.color.muted}>{active ? '●' : '○'}</Text>
}

// ── Session info section ───────────────────────────────────────────

function SessionInfoSection({ t, wide }: { t: Theme; wide: boolean }) {
  const ui = useStore($uiState)
  const info = ui.info

  const model = info?.model ?? ''
  const shortModel = model.split('/').pop() ?? model
  const sid = ui.sid ?? ''
  const shortSid = sid.length > 12 ? `…${sid.slice(-12)}` : sid
  const ctxUsed = ui.usage.context_used ?? 0
  const ctxMax = ui.usage.context_max ?? 0
  const ctxPct = ctxMax > 0 ? Math.round((ctxUsed / ctxMax) * 100) : 0

  return (
    <Box flexDirection="column" width={wide ? RAVEN_WIDTH : undefined}>
      <ThinDivider t={t} />

      <Box flexDirection="column" marginTop={1} rowGap={0}>
        {model && (
          <InfoLine
            label="model"
            value={shortModel}
            t={t}
          />
        )}

        {sid && (
          <InfoLine label="session" value={shortSid} t={t} />
        )}

        {ctxMax > 0 && (
          <InfoLine
            label="context"
            value={`${(ctxUsed / 1000).toFixed(0)}K/${(ctxMax / 1000).toFixed(0)}K (${ctxPct}%)`}
            t={t}
          />
        )}
      </Box>

      <ThinDivider t={t} />

      <Box marginTop={1}>
        <StatusDot active={ui.busy} t={t} />
        <Text color={t.color.muted}>
          {' '}
          {ui.busy ? 'processing' : ui.status}
        </Text>
      </Box>

      {ui.liveSessionCount > 0 && (
        <Text color={t.color.muted}>
          <Text color={t.color.sessionLabel}>sessions </Text>
          {ui.liveSessionCount}
        </Text>
      )}
    </Box>
  )
}

// ── Main SidePanel component ──────────────────────────────────────

interface RavenSidePanelProps {
  width: number
}

export function RavenSidePanel({ width }: RavenSidePanelProps) {
  const ui = useStore($uiState)
  const t = ui.theme

  // Determine if we have enough width for the full raven art
  const showRaven = width >= RAVEN_WIDTH + 4
  const showLabels = width >= 24

  return (
    <Box
      borderColor={t.color.border}
      borderStyle={KRAKEN_BORDER}
      flexDirection="column"
      minWidth={16}
      paddingX={1}
      paddingY={1}
      width={width}
    >
      {/* Panel title */}
      <Box justifyContent="center">
        <Text bold color={t.color.accent}>
          {' ▓ the raven ▓'}
        </Text>
      </Box>

      <ThinDivider t={t} />

      {/* Raven ASCII art */}
      {showRaven && (
        <Box justifyContent="center" marginBottom={1} marginTop={1}>
          <RavenArt t={t} />
        </Box>
      )}

      {/* Session info */}
      {showLabels && (
        <Box flexDirection="column">
          <SessionInfoSection t={t} wide={showRaven} />
        </Box>
      )}

      {/* Footer */}
      <Divider t={t} />

      <Box justifyContent="center" marginTop={1}>
        <Text color={t.color.muted}>
          {'nevermore'}
        </Text>
      </Box>
    </Box>
  )
}
