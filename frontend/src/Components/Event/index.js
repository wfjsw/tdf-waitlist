import React from "react";

import { AuthContext, EventContext, WaitlistContext } from "../../contexts";
import { useTranslation } from "react-i18next";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faBell, faBellSlash } from "@fortawesome/pro-solid-svg-icons";
import { Modal } from "../Modal";
import { Button } from "../Form";
import { Box } from "../Box";

import soundFile from "./bell-ringing-04.mp3";
const storageKey = "EventNotifierSettings";

function handleMessage(event, authContext, waitlistContext) {
  const message = JSON.parse(event.data);
  if (window.Notification && Notification.permission === "granted") {
    if (message.permission && !authContext.access[message.permission]) return;
    if (message.waitlist_id && waitlistContext && waitlistContext.active !== message.waitlist_id) return;
    const options = { silent: true };
    if (message.tag) {
      options.tag = message.tag;
      options.renotify = true;
    }
    new Notification(message.message, options);
  } else if (window.Notification && Notification.permission === "default") {
    // RIP, we didn't ask for permission first!?
    Notification.requestPermission();
  }
}

export function EventNotifier() {
  const [modalOpen, setModalOpen] = React.useState(false);
  const [isPlaying, setIsPlaying] = React.useState(false);
  const authContext = React.useContext(AuthContext);
  const eventContext = React.useContext(EventContext);
  const waitlistContext = React.useContext(WaitlistContext);
  const { t } = useTranslation('notification');
  const playerRef = React.useRef(null);

  const [settings, setSettings] = React.useState(() => {
    if (window.localStorage && window.localStorage.getItem(storageKey)) {
      return JSON.parse(window.localStorage.getItem(storageKey));
    }
    return {};
  });
  React.useEffect(() => {
    // Persist to localStorage.
    if (window.localStorage) {
      window.localStorage.setItem(storageKey, JSON.stringify(settings));
    }
  }, [settings]);

  React.useEffect(() => {
    if (!playerRef) return;
    if (isPlaying) {
      playerRef.current.play();
    } else {
      playerRef.current.pause();
    }
  }, [isPlaying, playerRef]);
  const handleWakeup = React.useCallback(
    (event) => {
      if (window.Notification && Notification.permission === "granted") {
        new Notification(event.data);
      } else if (window.Notification && Notification.permission === "default") {
        // RIP, we didn't ask for permission first!?
        Notification.requestPermission();
      }

      if (settings.enableSound && playerRef.current) {
        setIsPlaying(event.data);
      }
    },
    [settings]
  );

  React.useEffect(() => {
    if (eventContext == null) {
      return;
    }

    eventContext.addEventListener("wakeup", handleWakeup);
    const msgHandler = (event) => handleMessage(event, authContext, waitlistContext);
    eventContext.addEventListener("message", msgHandler);
    return () => {
      eventContext.removeEventListener("wakeup", handleWakeup);
      eventContext.removeEventListener("message", msgHandler);
    };
  }, [handleWakeup, authContext, eventContext, waitlistContext]);

  return (
    <>
      <Modal open={modalOpen} setOpen={setModalOpen}>
        <Box>
          <p>
            <label>
              <input
                checked={settings.enableSound}
                onChange={(evt) => setSettings({ ...settings, enableSound: !!evt.target.checked })}
                type="checkbox"
              />{" "}
              {t('enable_sound')}
            </label>
          </p>
          <Button onClick={(evt) => handleWakeup({ data: "This is a test alert" })}>
            {t('send_test')}
          </Button>
        </Box>
      </Modal>
      <Modal open={isPlaying} setOpen={setIsPlaying}>
        <Box>
          <p>{isPlaying}</p>
          <Button onClick={(evt) => setIsPlaying(false)} variant="success">
            OK
          </Button>
          <audio ref={playerRef} loop>
            <source src={soundFile} type="audio/mp3" />
          </audio>
        </Box>
      </Modal>
      <Button
        onClick={(evt) => setModalOpen(true)}
        title={
          settings.enableSound
            ? t('sound_enabled')
            : t('sound_disabled')
        }
      >
        <FontAwesomeIcon fixedWidth icon={settings.enableSound ? faBell : faBellSlash} />
      </Button>
    </>
  );
}
