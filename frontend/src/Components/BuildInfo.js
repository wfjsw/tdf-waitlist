import React from "react";
import styled from "styled-components";
import { AuthContext } from "../contexts";

const Footer = styled.footer`
  margin: 2em 0 1em;
  text-align: center;
  font-size: 0.75em;
  color: ${(props) => props.theme.colors.accent4};
`;

// APP_VERSION (from package.json) and RELEASE (build timestamp) are injected at
// build time by webpack's DefinePlugin, see config-overrides.js. They are only
// defined inside a webpack build, so reading them directly would throw a
// ReferenceError anywhere else - under Jest, for example.
const version = typeof APP_VERSION === "undefined" ? null : APP_VERSION;
const release = typeof RELEASE === "undefined" ? null : RELEASE;

// Deliberately not Util/time.js formatDatetime, which uses toLocaleString and so
// varies by browser locale. A build identifier that support asks someone to read
// back needs to look the same for everyone.
// Built from UTC components rather than slicing toISOString(), because years
// outside 0000-9999 serialise to an expanded format and would shift a fixed slice.
function formatBuildTime(value) {
  const seconds = Number(value);
  if (!Number.isFinite(seconds)) {
    return null;
  }

  const builtAt = new Date(seconds * 1000);
  if (Number.isNaN(builtAt.getTime())) {
    return null;
  }

  const pad = (n) => String(n).padStart(2, "0");
  const date = `${builtAt.getUTCFullYear()}-${pad(builtAt.getUTCMonth() + 1)}-${pad(
    builtAt.getUTCDate()
  )}`;
  return `${date} ${pad(builtAt.getUTCHours())}:${pad(builtAt.getUTCMinutes())} UTC`;
}

export function BuildInfo() {
  const authContext = React.useContext(AuthContext);

  // Only useful to the people who triage problems, so keep it out of the way of
  // everyone else. fleet-view is the broadest "is an FC" key, matching FCMenu.
  if (!authContext || !authContext.access || !authContext.access["fleet-view"]) {
    return null;
  }

  if (!version && !release) {
    return <Footer>Development build</Footer>;
  }

  // Fall back to showing the raw value rather than claiming a development build,
  // which would be actively misleading on a deployed site.
  const builtAt = release === null ? null : formatBuildTime(release) || String(release);

  return (
    <Footer>
      {version ? `v${version}` : "Version unknown"}
      {builtAt ? ` · built ${builtAt}` : ""}
    </Footer>
  );
}
