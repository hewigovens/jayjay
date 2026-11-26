/** @jsxRuntime automatic */

export function Icon({
  d,
  size = 14,
  className = "",
}: {
  d: string;
  size?: number;
  className?: string;
}) {
  return (
    <svg
      className={`jj-icon ${className}`}
      viewBox="0 0 16 16"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={d} />
    </svg>
  );
}

export const TreeIcon = ({ size }: { size?: number }) => (
  <Icon size={size} d="M2.5 3.5h11M5.5 8h8M5.5 12.5h8M2.5 6.5v6" />
);
export const ListIcon = () => (
  <Icon d="M5 3.5h8.5M5 8h8.5M5 12.5h8.5M2.25 3.5h.5M2.25 8h.5M2.25 12.5h.5" />
);
export const KeyboardIcon = () => (
  <Icon d="M1 3h14v10H1ZM4 6.25h.01M6.67 6.25h.01M9.33 6.25h.01M12 6.25h.01M5.25 9.75h5.5" />
);
export const SearchIcon = () => <Icon d="M7 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10ZM10.6 10.6l3.4 3.4" />;
export const HideReviewedIcon = ({ size }: { size?: number }) => (
  <Icon
    size={size}
    d="M2 8s2.2-4.5 6-4.5S14 8 14 8s-2.2 4.5-6 4.5S2 8 2 8ZM8 9.75a1.75 1.75 0 1 0 0-3.5 1.75 1.75 0 0 0 0 3.5ZM2.5 13.5l11-11"
  />
);
// Octicons file-directory, the folder the GPUI shell draws.
export const FolderIcon = () => (
  <svg
    className="jj-icon"
    viewBox="0 0 16 16"
    width="15"
    height="15"
    fill="currentColor"
    aria-hidden="true"
  >
    <path d="M0 2.75C0 1.784.784 1 1.75 1H5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1h6.75c.966 0 1.75.784 1.75 1.75v8.5A1.75 1.75 0 0 1 14.25 15H1.75A1.75 1.75 0 0 1 0 13.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 0 0 .25-.25v-8.5a.25.25 0 0 0-.25-.25H7.5c-.55 0-1.07-.26-1.4-.7l-.9-1.2a.25.25 0 0 0-.2-.1Z" />
  </svg>
);
export const SidebarIcon = () => <Icon d="M2.5 3.5h11v9h-11zM6 3.5v9" />;
export const ReviewModeIcon = () => (
  <Icon d="M2 4.5 3.5 6 6 3.5M8.5 4.75H14M2 11l1.5 1.5L6 10M8.5 11.25H14" />
);
export const ExpandAllIcon = () => (
  <Icon
    size={12}
    d="M2.5 5.5h11v5h-11zM8 1.5v2.5M6.5 2.75 8 1.25l1.5 1.5M8 14.5V12M6.5 13.25 8 14.75l1.5-1.5"
  />
);
export const CollapseAllIcon = () => (
  <Icon
    size={12}
    d="M2.5 5.5h11v5h-11zM8 .75v3M6.5 2.5 8 4l1.5-1.5M8 15.25v-3M6.5 13.5 8 12l1.5 1.5"
  />
);
export const OneListIcon = () => <Icon d="M2.5 2.5h11v4.5h-11zM2.5 9h11v4.5h-11z" />;
export const ChevronIcon = ({ open }: { open: boolean }) => (
  <Icon size={10} d={open ? "M4 6l4 4 4-4" : "M6 4l4 4-4 4"} />
);
export const CommitIcon = () => (
  <Icon d="M8 10.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5ZM1.5 8h4M10.5 8h4" />
);
export const FileIcon = () => <Icon d="M4 1.75h5.25L12.5 5v9.25H4V1.75ZM9 1.75V5.25h3.5" />;
export const CopyIcon = () => (
  <Icon size={12} d="M5.5 5.5V2.75h7.75v7.75H10.5M2.75 5.5h7.75v7.75H2.75V5.5Z" />
);
export const CheckIcon = () => <Icon size={12} d="M3 8.5l3 3 7-7" />;
export const UnifiedIcon = () => (
  <Icon size={12} d="M2.5 3.5h11M2.5 6.5h11M2.5 9.5h11M2.5 12.5h11" />
);
export const SplitIcon = () => <Icon size={12} d="M2 3h12v10H2V3ZM8 3v10" />;
export const WhitespaceIcon = () => (
  <Icon size={12} d="M2 11.5h3M6.5 11.5h3M11 11.5h3M4 4.5h8M8 4.5v4.5" />
);
export const AllLinesIcon = () => <Icon size={12} d="M8 2v12M5 5l3-3 3 3M5 11l3 3 3-3" />;
export const NoteIcon = () => (
  <Icon
    size={12}
    d="M2.5 3.25c0-.55.45-1 1-1h9c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H7l-3 2.5v-2.5h-.5c-.55 0-1-.45-1-1v-6.5ZM5 5.5h6M5 8h4"
  />
);
export const CloseIcon = () => <Icon size={12} d="M4 4l8 8M12 4l-8 8" />;
