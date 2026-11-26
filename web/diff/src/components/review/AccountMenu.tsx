import { Picker } from "@jayjay/review-web/react";
import type { Labels } from "@jayjay/review-web/types";
import { AccountIcon, SignOutIcon } from "../icons";

export function AccountMenu({ labels, signOut }: { labels: Labels; signOut: () => void }) {
  return (
    <Picker
      label={labels.pullAccount}
      button={<AccountIcon />}
      items={[
        {
          key: "sign-out",
          current: false,
          pick: signOut,
          content: (
            <>
              <SignOutIcon />
              <span className="jj-picker-title">{labels.pullSignOut}</span>
            </>
          ),
        },
      ]}
    />
  );
}
