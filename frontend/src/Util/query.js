import React from "react";
import { useNavigate, useLocation } from "react-router";

export function useQuery() {
  const location = useLocation();
  const navigate = useNavigate();
  const query = location.search;

  const parsed = React.useMemo(() => {
    const queryParams = new URLSearchParams(query);
    var result = {};
    for (const [key, value] of queryParams) {
      result[key] = value;
    }
    return result;
  }, [query]);

  const setParam = React.useCallback(
    (key, value, replace) => {
      const queryParams = new URLSearchParams(query);
      if (value === null) {
        queryParams.delete(key);
      } else {
        queryParams.set(key, value);
      }
      navigate({
        search: queryParams.toString(),
      }, {replace: !!replace});
    },
    [query, navigate]
  );

  return [parsed, setParam];
}
