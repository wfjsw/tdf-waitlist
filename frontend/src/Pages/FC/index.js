import React from "react";
import { Route, Routes } from "react-router-dom";
import { AuthContext } from "../../contexts";
import { BanRoutes } from "./Bans";
import { ACLRoutes } from "./ACL";
import { Fleet, FleetRegister } from "./Fleet";
import { Search } from "./Search";
import { Statistics } from "./Statistics";
import { FleetCompHistory } from "./FleetCompHistory";
import { NoteAdd } from "./NoteAdd";
import { FCMenu, GuideFC } from "./FCMenu";

export function FCRoutes() {
  const authContext = React.useContext(AuthContext);
  return (
    <Routes>
      <Route path="bans/*" element={<BanRoutes />} />
      <Route path="acl/*" element={<ACLRoutes />} />

      <Route path="/" element={<FCMenu />} />
      <Route path="stats" element={<Statistics />} />
      <Route path="fleet" element={<Fleet />} />
      <Route path="fleet/register" element={<FleetRegister />} />
      <Route path="search" element={<Search />} />
      <Route path="fleet-comp-history" element={<FleetCompHistory />} />
      <Route path="notes/add" element={<NoteAdd />} />
      {authContext.access["stats-view"] && (
        <Route path="documentation" element={<GuideFC />} />
      )}
      {authContext.access["fleet-view"] && (
        <Route path="trainee" element={<GuideFC />} />
      )}
    </Routes>
  );
}
