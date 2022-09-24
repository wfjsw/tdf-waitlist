import React from "react";
import { ToastContext, AuthContext, WaitlistContext } from "../../contexts";
import { addToast } from "../../Components/Toast";
import { apiCall, errorToaster, useApi } from "../../api";
import { Button, Buttons, InputGroup, NavButton, Textarea } from "../../Components/Form";
// import { useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Content, PageTitle } from "../../Components/Page";
import { FitDisplay, ImplantDisplay } from "../../Components/FitDisplay";
import { Box } from "../../Components/Box";
import { Modal } from "../../Components/Modal";
import { InfoAnnouncement } from "../../Components/Announcement";

import howToX from "./howtox.png";

const exampleFit = String.raw`
[Vindicator, Vindicator]
Shadow Serpentis Damage Control
Centum A-Type Multispectrum Energized Membrane
Centum A-Type Multispectrum Energized Membrane
Federation Navy Magnetic Field Stabilizer
Federation Navy Magnetic Field Stabilizer
Federation Navy Magnetic Field Stabilizer
Federation Navy Magnetic Field Stabilizer

Core X-Type 500MN Microwarpdrive
Federation Navy Stasis Webifier
Federation Navy Stasis Webifier
Federation Navy Stasis Webifier
Large Micro Jump Drive

Neutron Blaster Cannon II
Neutron Blaster Cannon II
Neutron Blaster Cannon II
Neutron Blaster Cannon II
Neutron Blaster Cannon II
Neutron Blaster Cannon II
Neutron Blaster Cannon II
Neutron Blaster Cannon II

Large Hybrid Locus Coordinator II
Large Explosive Armor Reinforcer II
Large Hyperspatial Velocity Optimizer II



'Augmented' Ogre x5

Null L x10000
Nanite Repair Paste x2000
Void L x20000
Agency 'Pyrolancea' DB7 Dose III x1
Standard Drop Booster x10
Quafe Zero Classic x1
Agency 'Pyrolancea' DB3 Dose I x10
Agency 'Pyrolancea' DB5 Dose II x10
`.trim();

async function xUp({ character, eft, toastContext, waitlist_id, alt }) {
  await apiCall("/api/waitlist/xup", {
    json: { eft: eft, character_id: character, waitlist_id: parseInt(waitlist_id), is_alt: alt },
  });

  addToast(toastContext, {
    title: "Added to waitlist.",
    message: "Your X has been added to the waitlist!",
    variant: "success",
  });

  if (window.Notification) {
    Notification.requestPermission();
  }
}

export function Xup() {
  const toastContext = React.useContext(ToastContext);
  const authContext = React.useContext(AuthContext);
  const waitlistContext = React.useContext(WaitlistContext);
  const { t } = useTranslation('x_up');
  const [eft, setEft] = React.useState("");
  const [isSubmitting, setIsSubmitting] = React.useState(false);
  const [reviewOpen, setReviewOpen] = React.useState(false);
  const [alt, setAlt] = React.useState(false);
  const [implants] = useApi(`/api/implants?character_id=${authContext.current.id}`);

  const handleChange = () => {
    setAlt(!alt);
  };

  const waitlist_id = waitlistContext && waitlistContext.active;
  if (!waitlist_id) {
    return <em>Missing waitlist information</em>;
  }

  const waitlist = waitlistContext.available.find((wl) => wl.id === waitlist_id);
  
  if (!waitlist.open) {
    return <em>Waitlist is closed</em>;
  }

  return (
    <>
      {reviewOpen && (
        <Modal open={true} setOpen={(evt) => null}>
          <Box>
            <XupCheck waitlistId={waitlist_id} setOpen={setReviewOpen} />
          </Box>
        </Modal>
      )}

      <div style={{ display: "flex" }}>
        <Content style={{ flex: 1 }}>
          <h2>{t('x_up_with_fit')}</h2>
          <Textarea
            placeholder={exampleFit}
            rows={15}
            onChange={(evt) => setEft(evt.target.value)}
            value={eft}
            style={{ width: "100%", marginBottom: "1em" }}
          />
          <InfoAnnouncement id={3} />
          <div>
            <label>
              <input type="checkbox" checked={alt} onChange={handleChange} />
              {t('alt_checkbox')}
            </label>
          </div>

          <InputGroup>
            <Button static>{authContext.current.name}</Button>
            <Button
              variant="success"
              onClick={(evt) => {
                setIsSubmitting(true);
                errorToaster(
                  toastContext,
                  xUp({
                    character: authContext.current.id,
                    eft,
                    toastContext,
                    waitlist_id,
                    alt,
                  }).then((evt) => setReviewOpen(true))
                ).finally((evt) => setIsSubmitting(false));
              }}
              disabled={eft.trim().length < 50 || !eft.startsWith("[") || isSubmitting}
            >
              {t('x_up')}
            </Button>
          </InputGroup>

          <h2>How to X up?</h2>
          <img
            src={howToX}
            alt="On the bottom left of the Fitting window you will find a copy button"
          />
        </Content>
        <Box style={{ flex: 1 }}>
          {implants ? (
            <ImplantDisplay
              implants={implants.implants}
              name={`${authContext.current.name}'s capsule`}
            />
          ) : null}
        </Box>
      </div>
    </>
  );
}

function XupCheck({ waitlistId, setOpen }) {
  const authContext = React.useContext(AuthContext);
  const { t } = useTranslation('x_up');
  const [xupData] = useApi(`/api/waitlist?waitlist_id=${waitlistId}`);

  if (!xupData) {
    return <em>Loading</em>;
  }

  // const myEntry = _.find(
  //   xupData.waitlist,
  //   (entry) => entry.character && entry.character.id === authContext.current.id
  // );

  const fit = xupData.waitlist
    .find(
      (ent) =>
        ent &&
        ent.fits &&
        ent.fits.some((fit) => fit.character && fit.character.id === authContext.current.id)
    )
    .fits.find((fit) => fit.character && fit.character.id === authContext.current.id);

  return (
    <>
      <PageTitle>{t("fit_review")}</PageTitle>
      <em>
        {t("fit_review_desc")}
      </em>
      <Box key={fit.id}>
        <FitDisplay fit={fit} />
      </Box>
      <Buttons>
        <NavButton variant="primary" to={`/waitlist?wl=${waitlistId}`}>
          {t("yes_looks_good")}
        </NavButton>
        <Button variant="secondary" onClick={(evt) => setOpen(false)}>
          {t("no_go_back_update_my_fit")}
        </Button>
      </Buttons>
    </>
  );
}
