/** @jsxRuntime automatic */

function Icon({ d, size = 14 }: { d: string; size?: number }) {
  return <svg className="icon" viewBox="0 0 16 16" width={size} height={size} fill="none" stroke="currentColor" strokeWidth="1.4"
    strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={d} /></svg>;
}

export const TreeIcon = () => <Icon d="M2.5 3.5h11M5.5 8h8M5.5 12.5h8M2.5 6.5v6" />;
export const ListIcon = () => <Icon d="M5 3.5h8.5M5 8h8.5M5 12.5h8.5M2.25 3.5h.5M2.25 8h.5M2.25 12.5h.5" />;
export const SearchIcon = () => <Icon d="M7 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10ZM10.6 10.6l3.4 3.4" />;
export const HideReviewedIcon = () => <Icon d="M2 8s2.2-4.5 6-4.5S14 8 14 8s-2.2 4.5-6 4.5S2 8 2 8ZM8 9.75a1.75 1.75 0 1 0 0-3.5 1.75 1.75 0 0 0 0 3.5ZM2.5 13.5l11-11" />;
export const FolderIcon = () => <Icon size={12} d="M1.75 4.25c0-.55.45-1 1-1h3.2l1.5 1.5h5.8c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H2.75c-.55 0-1-.45-1-1v-8Z" />;
export const ChevronIcon = ({ open }: { open: boolean }) => <Icon size={10} d={open ? "M4 6l4 4 4-4" : "M6 4l4 4-4 4"} />;
export const FileIcon = () => <Icon d="M4 1.75h5.25L12.5 5v9.25H4V1.75ZM9 1.75V5.25h3.5" />;
export const CopyIcon = () => <Icon size={12} d="M5.5 5.5V2.75h7.75v7.75H10.5M2.75 5.5h7.75v7.75H2.75V5.5Z" />;
export const CheckIcon = () => <Icon size={12} d="M3 8.5l3 3 7-7" />;
export const UnifiedIcon = () => <Icon size={12} d="M2.5 3.5h11M2.5 6.5h11M2.5 9.5h11M2.5 12.5h11" />;
export const SplitIcon = () => <Icon size={12} d="M2 3h12v10H2V3ZM8 3v10" />;
export const WhitespaceIcon = () => <Icon size={12} d="M2 11.5h3M6.5 11.5h3M11 11.5h3M4 4.5h8M8 4.5v4.5" />;
export const AllLinesIcon = () => <Icon size={12} d="M8 2v12M5 5l3-3 3 3M5 11l3 3 3-3" />;
export const NoteIcon = () => <Icon size={12} d="M2.5 3.25c0-.55.45-1 1-1h9c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H7l-3 2.5v-2.5h-.5c-.55 0-1-.45-1-1v-6.5ZM5 5.5h6M5 8h4" />;
export const ExpandIcon = ({ expanded }: { expanded: boolean }) => <Icon size={12} d={expanded ? "M8 2v4.5M5.5 4 8 6.5 10.5 4M2.5 8h11M8 14V9.5M5.5 12 8 9.5l2.5 2.5" : "M8 6.5V2M5.5 4.5 8 2l2.5 2.5M2.5 8h11M8 9.5V14M5.5 11.5 8 14l2.5-2.5"} />;
export const CloseIcon =() => <Icon size={12} d="M4 4l8 8M12 4l-8 8" />;
