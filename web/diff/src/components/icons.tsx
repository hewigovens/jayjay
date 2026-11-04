import { Icon } from "@jayjay/review-web/react";

interface IconProps {
  size?: number;
}

export const GitHubIcon = ({ size = 14 }: IconProps) => (
  <svg
    className="jj-icon"
    width={size}
    height={size}
    viewBox="0 0 16 16"
    fill="currentColor"
    aria-hidden="true"
  >
    <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z" />
  </svg>
);
export const ShareIcon = ({ size }: IconProps) => (
  <Icon size={size} d="M8 10V2.5M5.5 5 8 2.5 10.5 5M5 7.5H3.5v6h9v-6H11" />
);
export const SignOutIcon = ({ size }: IconProps) => (
  <Icon size={size} d="M6.5 2.5h-3v11h3M10.5 5l3 3-3 3M13.5 8h-7" />
);
export const SyncIcon = ({ size }: IconProps) => (
  <Icon
    size={size}
    d="M12.9 6.5A5 5 0 0 0 3.5 5M3 2.5V5h2.5M3.1 9.5a5 5 0 0 0 9.4 1.5M13 13.5V11h-2.5"
  />
);
export const CommitIcon = ({ size }: IconProps) => (
  <Icon size={size} d="M8 10.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5ZM1.5 8h4M10.5 8h4" />
);
export const StackIcon = ({ size }: IconProps) => (
  <Icon size={size} d="M8 2l6 3-6 3-6-3 6-3ZM2 8l6 3 6-3M2 11l6 3 6-3" />
);
export const CheckIcon = ({ size }: IconProps) => <Icon size={size} d="M3 8.5l3 3 7-7" />;
export const FileIcon = ({ size }: IconProps) => (
  <Icon size={size} d="M4 1.75h5.25L12.5 5v9.25H4V1.75ZM9 1.75V5.25h3.5" />
);
