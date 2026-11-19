import { useEffect, useState } from "react";

/** True once `pending` has lasted longer than a file switch normally takes, so quick work shows no loading state. */
export function useOverdue(pending: boolean) {
  const [overdue, setOverdue] = useState(false);
  useEffect(() => {
    if (!pending) return setOverdue(false);
    const timer = setTimeout(() => setOverdue(true), 300);
    return () => clearTimeout(timer);
  }, [pending]);
  return pending && overdue;
}
