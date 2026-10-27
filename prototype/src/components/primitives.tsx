import React from 'react'
import type { AgentStatus } from '../types'
import { STATUS_DOT, STATUS_RING } from '../data'
import { I } from '../icons'
import { usePapago } from '../store'

export const Icon = ({ g, className = '' }: { g: string; className?: string }) => (
  <span aria-hidden className={`nf leading-none ${className}`}>
    {g}
  </span>
)

// Status dot — see the semantics comment on STATUS_DOT (data.ts).
// `ring` draws the hollow orchestrator form (same status colour, no fill).
export const Dot = ({ status, ring = false, className = '' }: { status: AgentStatus; ring?: boolean; className?: string }) =>
  ring ? (
    <span className={`inline-block w-2.5 h-2.5 rounded-full shrink-0 border-2 bg-transparent ${STATUS_RING[status] || 'border-mut'} ${status === 'running' ? 'animate-pulse' : ''} ${className}`} />
  ) : (
    <span className={`inline-block w-2.5 h-2.5 rounded-full shrink-0 ${STATUS_DOT[status] || 'bg-mut'} ${className}`} />
  )

export const Popover = ({
  open,
  onClose,
  anchor = 'right',
  children,
  width = '15rem',
}: {
  open: boolean
  onClose: () => void
  anchor?: 'left' | 'right'
  children: React.ReactNode
  width?: string
}) => {
  if (!open) return null
  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} />
      <div
        className={`absolute z-50 bottom-[calc(100%+6px)] ${anchor === 'right' ? 'right-0' : 'left-0'} bg-panel border border-white/10 rounded-md shadow-2xl p-2`}
        style={{ width }}
      >
        {children}
      </div>
    </>
  )
}

export const Seg = ({
  children,
  onClick,
  tone = 'default',
  title,
}: {
  children: React.ReactNode
  onClick?: () => void
  tone?: 'default' | 'muted' | 'accent' | 'warn' | 'err' | 'brand'
  title?: string
}) => {
  const color =
    tone === 'accent'
      ? 'text-acc'
      : tone === 'warn'
        ? 'text-warn'
        : tone === 'err'
          ? 'text-err'
          : tone === 'muted'
            ? 'text-mut/70'
            : tone === 'brand'
              ? 'text-acc bg-white/[0.07]'
              : 'text-mut'
  const interactive = onClick ? 'cursor-pointer hover:bg-white/[0.07] hover:text-ink' : ''
  const Tag = onClick ? 'button' : 'div'
  return (
    <Tag title={title} onClick={onClick} className={`h-full px-3 flex items-center gap-1.5 text-xs whitespace-nowrap border-l border-line/40 first:border-l-0 ${color} ${interactive}`}>
      {children}
    </Tag>
  )
}

export const MenuItem = ({
  active,
  onClick,
  children,
}: {
  active?: boolean
  onClick?: () => void
  children: React.ReactNode
}) => (
  <button
    onClick={onClick}
    className={`w-full flex items-center gap-2 px-2 py-2 rounded text-xs text-left ${active ? 'text-acc' : 'text-ink hover:bg-white/[0.07]'}`}
  >
    <span className="w-4 shrink-0">{active ? <Icon g={I.check} /> : ''}</span>
    <span className="truncate">{children}</span>
  </button>
)


// Design decision (Synth, 2026-10-27): status-bar items and tree rows are shared
// shells — every trigger+popover goes through StatusMenu, every tree row through
// TreeRow. Extract a component the moment the pattern appears twice (AGENTS.md).
export const StatusMenu = ({
  id,
  icon,
  iconClassName = '',
  label,
  title,
  width = '16rem',
  children,
}: {
  id: string
  icon?: string
  iconClassName?: string
  label: React.ReactNode
  title?: string
  width?: string
  children: (close: () => void) => React.ReactNode
}) => {
  const open = usePapago((s) => !!s.popovers[id])
  const togglePopover = usePapago((s) => s.togglePopover)
  const close = () => togglePopover(id)
  return (
    <span className="relative flex">
      <Seg onClick={() => togglePopover(id)} title={title ?? id}>
        {icon && <Icon g={icon} className={iconClassName} />}
        {label}
        <Icon g={I.caretDown} className="text-xs" />
      </Seg>
      <Popover open={open} onClose={close} anchor="left" width={width}>
        {children(close)}
      </Popover>
    </span>
  )
}

export const TreeRow = ({
  indent = 1,
  selected = false,
  onClick,
  onCaret,
  open = true,
  title,
  children,
}: {
  indent?: 0 | 1 | 2 | 3
  selected?: boolean
  onClick?: () => void
  onCaret?: () => void
  open?: boolean
  title?: string
  children: React.ReactNode
}) => {
  const pads = ['px-2', 'pl-6 pr-2', 'pl-10 pr-2', 'pl-14 pr-2']
  return (
    <button
      onClick={onClick}
      title={title}
      className={`w-full flex items-center gap-2 py-1.5 text-left ${pads[indent]} ${selected ? 'bg-white/[0.08]' : 'hover:bg-white/[0.06]'}`}
    >
      {onCaret && (
        // Fold toggle is its own hit target; the row click selects the row's agent.
        <span
          onClick={(e) => {
            e.stopPropagation()
            onCaret()
          }}
          className="text-xs text-mut/60 hover:text-ink shrink-0 -ml-1 px-0.5"
        >
          {open ? <Icon g={I.caretDown} /> : <Icon g={I.caretRight} />}
        </span>
      )}
      {children}
    </button>
  )
}

// Icon-only toolbar button (window controls, panel header actions): one shell so
// the size/hover/tone behaviour never drifts between surfaces.
export const IconButton = ({
  g,
  title,
  onClick,
  tone = 'mut',
  className = '',
}: {
  g: string
  title: string
  onClick?: () => void
  tone?: 'mut' | 'err'
  className?: string
}) => (
  <button
    title={title}
    onClick={onClick}
    className={`h-7 w-7 flex items-center justify-center rounded hover:bg-white/[0.07] ${
      tone === 'err' ? 'text-mut hover:text-err' : 'text-mut hover:text-ink'
    } ${className}`}
  >
    <Icon g={g} />
  </button>
)

// Tree connector: the hairline that ties a child row to its parent. A vertical
// runs at the parent caret's centre (left-2.5 = 10px) down the group; on the last
// row it stops at the row's midline and turns right into an elbow that meets the
// child's status dot (24px), wrapping the group instead of trailing off.
// Render it AFTER the row inside a `relative` wrapper so the row's hover/selected
// background never paints over the line.
export const TreeConnector = ({ last = false }: { last?: boolean }) => (
  <>
    <span className={`pointer-events-none absolute left-2.5 w-px bg-white/10 ${last ? 'top-0 h-1/2' : 'inset-y-0'}`} />
    {last && <span className="pointer-events-none absolute left-2.5 top-1/2 w-3.5 h-px bg-white/10" />}
  </>
)
