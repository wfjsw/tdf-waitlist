import { Route, Routes } from "react-router";
import { Skills } from "./Skills";

export function SkillRoutes() {
  return (
    <Routes>
      <Route path="/" element={<Skills />} />
    </Routes>
  );
}
