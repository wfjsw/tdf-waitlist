import React from "react";
import { AuthContext } from "../../contexts";
import { useLocation, useNavigate } from "react-router-dom";
import { PageTitle } from "../../Components/Page";
import { useApi } from "../../api";
import { useTranslation } from "react-i18next";
import { SkillDisplay } from "../../Components/SkillDisplay";

export function Skills() {
  const authContext = React.useContext(AuthContext);
  const { t } = useTranslation("skills");
  const queryParams = new URLSearchParams(useLocation().search);
  const navigate = useNavigate();

  var characterId = queryParams.get("character_id") || authContext.current.id;
  var ship = queryParams.get("ship") || "Vindicator";

  const [basicInfo] = useApi(`/api/pilot/info?character_id=${characterId}`);

  const setShip = (newShip) => {
    queryParams.set("ship", newShip);
    navigate({
      search: queryParams.toString(),
    });
  };

  return (
    <>
      <PageTitle>{basicInfo ? t('skills_for', {name: basicInfo.name}) : t('skills')}</PageTitle>
      <SkillDisplay characterId={characterId} ship={ship} setShip={setShip} />
    </>
  );
}
