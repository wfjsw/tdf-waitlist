import React from "react";
import styled, { ThemeContext } from "styled-components";
import { ToastContext, AuthContext } from "../../contexts";
import { apiCall, useApi, errorToaster } from "../../api";
import { NavLink } from "react-router-dom";
import { TimeDisplay } from "./TimeDisplay.js";
import { Badge, Shield, tagBadges } from "../../Components/Badge";
import { Modal } from "../../Components/Modal";
import { FitDisplay } from "../../Components/FitDisplay";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faTrashAlt,
  faCheck,
  faExternalLinkAlt,
  faBook,
  faPlus,
  faExclamationTriangle,
  faTimes,
  faSpinner,
} from "@fortawesome/free-solid-svg-icons";
import { sortBy, isFinite } from "lodash";

import egoldBadge from "../Guide/badges/egold.png";

import { SkillDisplay } from "../../Components/SkillDisplay";
import { Box } from "../../Components/Box";
import { Title } from "../../Components/Page";
import { Button, InputGroup } from "../../Components/Form";
import { useTranslation } from "react-i18next";
import { AltNumberDisplay } from "./AltNumberDisplay.js";

const badgeOrder = [
  "HQ-FC",
  "TRAINEE",
  "WEB",
  "BASTION",
  "LOGI",
  "RETIRED-LOGI",
  "AMULET",
  "WARPSPEED",
  "HYBRID",
  "ELITE",
  "ELITE-GOLD",
];

async function approveFit(id) {
  return await apiCall("/api/waitlist/approve", {
    json: { id: id },
  });
}

async function rejectFit(id, review_comment) {
  return await apiCall("/api/waitlist/reject", {
    json: { id, review_comment },
  });
}

async function removeFit(id) {
  return await apiCall("/api/waitlist/remove_fit", {
    json: { id: id },
  });
}

async function openWindow(target_id, character_id) {
  return await apiCall(`/api/open_window`, {
    json: { target_id, character_id },
  });
}

const XCardDOM = styled.div`
  border: solid 2px ${(props) => props.theme.colors[props.variant].color};
  background-color: ${(props) => props.theme.colors[props.variant].color};
  color: ${(props) => props.theme.colors[props.variant].text};
  border-radius: 5px;
  font-size: 0.9em;
  filter: drop-shadow(0px 4px 5px ${(props) => props.theme.colors.shadow});
  min-width: 245px;

  a {
    color: inherit;
    text-decoration: none;
  }
`;
XCardDOM.Head = styled.div`
  display: flex;
  padding: 0.2em 0.2em 0.3em 0.5em;
  align-items: center;
  a {
    font-weight: 600;
  }
`;
XCardDOM.Head.Badges = styled.div`
  margin-left: auto;
  display: flex;
  flex-shrink: 0;
  align-items: center;
  > * {
    margin-left: 0.25em;
  }
  > *:last-child {
    margin-left: 0.5em;
  }
  > span {
    display: flex;
    align-items: center;
  }
  img {
    height: 1.3em;
  }
`;
XCardDOM.Content = styled.div`
  display: flex;

  background-color: ${(props) => props.theme.colors.background};
  align-items: center;
  color: ${(props) => props.theme.colors.text};
  img {
    margin-right: 0.5em;
    align-self: flex-start;
  }
  a {
    align-items: center;
    display: flex;
    cursor: pointer;
    width: 100%;
  }
`;

XCardDOM.FooterGroup = styled.div`
  > *:last-child {
    border-radius: 0 0 4px 4px;
  }
`;
XCardDOM.Footer = styled.div`
  display: flex;
  background-color: ${(props) => props.theme.colors.accent1};
  color: ${(props) => props.theme.colors.text};
  text-align: center;
  a:hover {
    background-color: ${(props) => props.theme.colors.accent2};
    cursor: pointer;
  }
  > * {
    flex-grow: 1;
    flex-basis: 0;
  }
`;
XCardDOM.ReviewComment = styled.div`
  padding: 0.5em;
  margin: 0.5em;
  width: 100%;
  text-align: center;
  background-color: ${(props) => props.theme.colors.secondary.color};
  border-radius: 5px;
  color: ${(props) => props.theme.colors.secondary.text};
`;

function ShipDisplay({ fit, onAction }) {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [modalOpen, setModalOpen] = React.useState(false);
  const { t } = useTranslation("waitlist");

  const namePrefix = fit.character ? `${fit.character.name}` : "";
  if (fit.dna && fit.hull) {
    return (
      <>
        {modalOpen ? (
          <Modal open={true} setOpen={setModalOpen}>
            <Box>
              {authContext.access["waitlist-manage"] && (
                <InputGroup style={{ marginBottom: "1em" }}>
                  <Button
                    variant="success"
                    onClick={(evt) => {
                      setModalOpen(false);
                      errorToaster(toastContext, approveFit(fit.id)).then(onAction);
                    }}
                  >
                    {t("approve")}
                  </Button>
                  <Button
                    onClick={(evt) => {
                      var rejectionReason = prompt(
                        "Why is the fit being rejected? (Will be displayed to pilot)"
                      );
                      if (rejectionReason) {
                        setModalOpen(false);
                        errorToaster(toastContext, rejectFit(fit.id, rejectionReason)).then(
                          onAction
                        );
                      }
                    }}
                  >
                    {t("reject")}
                  </Button>
                </InputGroup>
              )}

              <FitDisplay fit={fit} />
              {fit.tags.includes("STARTER") ? (
                <>
                  <Title>Starter skills</Title>
                  <SkillDisplay
                    characterId={fit.character.id}
                    ship={fit.hull.name}
                    filterMin={true}
                  />
                </>
              ) : null}
            </Box>
          </Modal>
        ) : null}
        <a onClick={(evt) => setModalOpen(true)}>
          <img
            style={{ height: "44px" }}
            src={`https://images.evetech.net/types/${fit.hull.id}/icon?size=64`}
            alt={fit.hull.name}
          />
          <span style={{ flexShrink: 1 }}>
            {namePrefix} ({fit.hull.name})
          </span>
        </a>
      </>
    );
  } else if (fit.hull) {
    return (
      <>
        <img
          style={{ height: "44px" }}
          src={`https://images.evetech.net/types/${fit.hull.id}/icon?size=64`}
          alt={fit.hull.name}
        />
        <span>
          {namePrefix} ({fit.hull.name})
        </span>
      </>
    );
  } else {
    return (
      <>
        <img
          style={{ height: "44px" }}
          src={`https://images.evetech.net/types/28606/icon?size=64`}
          alt=""
        />
        <span>{namePrefix} (Ship)</span>
      </>
    );
  }
}

function SkillButton({ characterId, ship }) {
  const [onScreen, setOnScreen] = React.useState(false);
  const [chosenShip, setChosenShip] = React.useState(ship);

  return (
    <>
      <a title="Show skills" onClick={(evt) => setOnScreen(true)}>
        <FontAwesomeIcon icon={faBook} />
      </a>
      {onScreen && (
        <Modal open={true} setOpen={setOnScreen}>
          <Box>
            <SkillDisplay characterId={characterId} ship={chosenShip} setShip={setChosenShip} />
          </Box>
        </Modal>
      )}
    </>
  );
}

function NoteButton({ number }) {
  const theme = React.useContext(ThemeContext);
  return (
    <span style={{ verticalAlign: "middle" }}>
      <svg style={{ height: "1em" }} viewBox="0 0 50 50" xmlns="http://www.w3.org/2000/svg">
        <g>
          <circle
            style={{ verticalAlign: "middle", fill: theme.colors.text }}
            cy="25"
            cx="25"
            r="24"
          />
          <text
            style={{
              fontSize: "2.6em",
              fontWeight: "600",
              textAnchor: "middle",
              fill: theme.colors.accent1,
              textRendering: "geometricPrecision",
            }}
            x="25"
            y="38"
          >
            {number}
          </text>
        </g>
      </svg>
    </span>
  );
}

function InviteButton({ fitId, fcId, onAction }) {
  const themeContext = React.useContext(ThemeContext);
  const toastContext = React.useContext(ToastContext);
  const [state, setState] = React.useState(undefined);
  const { t } = useTranslation("waitlist");

  const colours = {
    success: themeContext?.colors?.success?.color,
    failed: themeContext?.colors?.danger?.color,
    // pending: themeContext?.colors?.primary?.color,
  };

  const onClick = async () => {
    setState("pending");

    errorToaster(
      toastContext,
      apiCall("/api/waitlist/invite", {
        json: { id: fitId, character_id: fcId },
      })
        .then(() => {
          setState("success");
          onAction();
        })
        .catch((err) => {
          setState("failed");
          throw err;
        })
    );
  };

  return (
    <a
      title={t("invite")}
      onClick={onClick}
      style={{
        cursor: state === "Pending" ? "not-allowed" : null,
        backgroundColor: colours[state] ?? null,
        transition: "background-color 300ms linear",
      }}
    >
      <FontAwesomeIcon
        fixedWidth
        icon={state !== "pending" ? faPlus : faSpinner}
        spin={state === "pending"}
      />
    </a>
  );
}


function PilotInformation({ characterId, authContext, id }) {
  const [notes] = useApi(
    authContext.access["notes-view"] ? `/api/notes?character_id=${characterId}` : null
  );
  if (!notes)
    return (
      <NavLink to={"/pilot?character_id=" + id}>
        <NoteButton number={0} />
      </NavLink>
    );
  var amount = Object.keys(notes.notes).length;
  var msg = "Pilot information";
  amount = amount > 9 ? "9+" : amount.toString();
  if (amount > 0) msg += "\nLast Note:\n" + notes.notes[amount - 1].note;
  return (
    <NavLink title={msg} to={"/pilot?character_id=" + id}>
      <NoteButton number={amount} />
    </NavLink>
  );
}

export function XCard({ entry, fit, fleetComposition, onAction }) {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const { t } = useTranslation("waitlist");

  const is_alt = fit.is_alt;
  const accountName = entry.character ? entry.character.name : "Name hidden";
  const tags = sortBy(fit.tags, function (item) {
    return badgeOrder.indexOf(item);
  });
  // var isSelf = entry.character && entry.character.id === authContext.account_id;
  // let isSelf = entry.character && authContext.characters.some((character) => character.id === entry.character.id);
  const isSelf = entry.character && entry.character.account_id === authContext.account_id;
  let tagText = [];
  let tagImages = [];
  tags.forEach((tag) => {
    if (tag === "ELITE-GOLD") {
      tagImages.push(<img key={tag} src={egoldBadge} alt={"Elite GOLD"} title={"Elite GOLD"} />);
    } else if (tag in tagBadges) {
      tagImages.push(
        <Shield
          key={tag}
          color={tagBadges[tag][0]}
          letter={tagBadges[tag][1]}
          title={tagBadges[tag][2]}
        />
      );
    } else {
      tagText.push(tag);
    }
  });

  const approvalFlag = fit.approved ? null : (
    <span title={t("pending_approval")}>
      <FontAwesomeIcon icon={faExclamationTriangle} />
    </span>
  );

  const altNumber = fleetComposition && fleetComposition.members ? fleetComposition.members.filter(
    (member) => {
      const memberId = member?.account_id ?? member?.id;
      const fitId = fit.character?.account_id ?? fit.character?.id;
      return memberId === fitId;
    }
  ).length : null;

  return (
    <XCardDOM
      variant={
        fit.approved
          ? isSelf
            ? "success"
            : "secondary"
          : fit.approved
          ? "secondary"
          : fit.review_comment
          ? "danger"
          : isSelf || authContext.access["waitlist-view"]
          ? "warning"
          : "secondary"
      }
    >
      <XCardDOM.Head>
        <span>{accountName}</span>
        <XCardDOM.Head.Badges>
          {tagImages}
          {approvalFlag}
          {altNumber !== null && <AltNumberDisplay altNumber={altNumber} />}
          <TimeDisplay relativeTo={entry.joined_at} isAlt={is_alt} />
        </XCardDOM.Head.Badges>
      </XCardDOM.Head>
      <XCardDOM.Content>
        <ShipDisplay fit={fit} onAction={onAction} />
      </XCardDOM.Content>
      <XCardDOM.Content>
        <div style={{ FlexWrap: "wrap" }}>
          {tagText.map((tag) => (
            <Badge key={tag}>{tag}</Badge>
          ))}
        </div>
      </XCardDOM.Content>
      {fit.review_comment ? (
        <XCardDOM.Content>
          <XCardDOM.ReviewComment>{fit.review_comment}</XCardDOM.ReviewComment>
        </XCardDOM.Content>
      ) : null}
      <XCardDOM.FooterGroup>
        <XCardDOM.Footer>
          {entry.can_remove ? (
            <a
              title={t("remove_xup")}
              onClick={(evt) => errorToaster(toastContext, removeFit(fit.id)).then(onAction)}
            >
              <FontAwesomeIcon icon={faTrashAlt} />
            </a>
          ) : null}
          {authContext.access["waitlist-view"] && (
            <a
              title={t("open_ingame_profile")}
              onClick={(evt) =>
                errorToaster(toastContext, openWindow(fit.character.id, authContext.current.id))
              }
            >
              <FontAwesomeIcon icon={faExternalLinkAlt} />
            </a>
          )}
          {authContext.access["skill-view"] && (
            <SkillButton characterId={fit.character.id} ship={fit.hull.name} />
          )}
          {authContext.access["pilot-view"] && (
            <PilotInformation
              characterId={fit.character.id}
              authContext={authContext}
              id={fit.character.id}
            />
          )}
          {isFinite(fit.hours_in_fleet) ? (
            <span title={t("hours_in_fleet")}>{fit.hours_in_fleet}h</span>
          ) : null}
          {authContext.access["waitlist-manage"] && (
            <a
              title={t("reject")}
              onClick={(evt) => {
                var rejectionReason = prompt(
                  "Why is the fit being rejected? (Will be displayed to pilot)"
                );
                if (rejectionReason) {
                  errorToaster(toastContext, rejectFit(fit.id, rejectionReason)).then(onAction);
                }
              }}
            >
              <FontAwesomeIcon icon={faTimes} />
            </a>
          )}
          {authContext.access["fleet-invite"] && fit.approved && (
            <InviteButton fitId={fit.id} fcId={authContext.current.id} onAction={onAction} />
          )}
          {authContext.access["waitlist-manage"] && !fit.approved && (
            <a
              title={t("approve")}
              onClick={(evt) => errorToaster(toastContext, approveFit(fit.id)).then(onAction)}
            >
              <FontAwesomeIcon icon={faCheck} />
            </a>
          )}
        </XCardDOM.Footer>
        {!is_alt && isFinite(fit.hours_in_fleet) && fit.hours_in_fleet < 1 && (
          <XCardDOM.Footer>
            <span>NEWBRO</span>
          </XCardDOM.Footer>
        )}
      </XCardDOM.FooterGroup>
    </XCardDOM>
  );
}
