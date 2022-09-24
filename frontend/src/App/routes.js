import React, {lazy} from "react";

import { Route, Routes as Switch } from "react-router-dom";
import * as Sentry from "@sentry/react";
import { AuthContext } from "../contexts";

import { SkillRoutes } from "../Pages/Skills";
// import { Plans } from "../Pages/Skills/Plans";
// import { Skills } from "../Pages/Skills/Skills";
import { Waitlist } from "../Pages/Waitlist";
import { Xup } from "../Pages/Xup";
import { Pilot } from "../Pages/Pilot";
import { Home } from "../Pages/Home";
import { Legal } from "../Pages/Legal";
// import { Fits } from "../Pages/Fits";
// import { Guide, /*GuideIndex*/, BadgeIndex } from "../Pages/Guide";
import { AuthRoutes } from "../Pages/Auth";
// import { ISKh, ISKhCalc } from "../Pages/ISKh";

// const Guide = lazy(() => import("../Pages/Guide/lazy.guide"));
const BadgeIndex = lazy(() => import("../Pages/Guide/lazy.badgeindex"));
const FCRoutes = lazy(() => import("../Pages/FC/lazy"));

const SentryRoutes = Sentry.withSentryReactRouterV6Routing(Switch)

function LoginRequired() {
  return <b>Login Required!</b>;
}

export function Routes() {
  const authContext = React.useContext(AuthContext);
  return (
    <SentryRoutes>
      <Route path="/" element={<Home />} />
      <Route path="/legal" element={<Legal />} />

      {/* <Route path="/guide" element={<GuideIndex />} /> */}
      {/* <Route path="/guide/:guideName" element={<Guide />} /> */}
      <Route path="/badges" element={<BadgeIndex />} />
      {/* <Route path="/isk-h/calc" element={<ISKhCalc />} />
      <Route path="/isk-h" element={<ISKh />} />
      <Route path="/fits" element={<Fits />} />
      <Route path="/skills/plans" element={<Plans />} />*/}
      <Route path="/xup" element={authContext ? <Xup /> : <LoginRequired />} />
      <Route path="/waitlist" element={authContext ? <Waitlist /> : <LoginRequired />} />
      <Route path="/pilot" element={<Pilot />} />

      {authContext && (
        <>
          <Route path="/fc/*" element={<FCRoutes />} />
          <Route path="/skills/*" element={<SkillRoutes />} />
        </>
      )}

      <Route path="/auth/*" element={<AuthRoutes />} />
    </SentryRoutes>
  );
}
