import React from "react";
import { NavLink } from "react-router-dom";
import { AuthContext, WaitlistContext } from "../contexts";
import logoImage from "./logo.png";
import styled from "styled-components";
import { InputGroup, Select, NavButton, AButton } from "../Components/Form";
import { EventNotifier } from "../Components/Event";
import { ThemeSelect } from "../Components/ThemeSelect";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRotate, faBug, faSpinner } from "@fortawesome/free-solid-svg-icons";
import { NavLinks, MobileNavButton, MobileNav } from "./Navigation";
import { useTranslation } from 'react-i18next';

const NavBar = styled.div`
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  padding: 1em;
  margin-bottom: 1em;
  @media (max-width: 480px) {
    padding: 0.2em;
    justify-content: space-between;
  }
`;
NavBar.Header = styled.div`
  display: flex;
  @media (max-width: 480px) {
    width: 100%;
    border-bottom: 3px solid;
    margin-bottom: 1em;
    padding-bottom: 0.2em;
  }
`;

NavBar.LogoLink = styled(NavLink).attrs((props) => ({
  className: props.isActive ? "active" : '',
}))`
  margin-right: 2em;
  flex-grow: 0;
  line-height: 0;
  @media (max-width: 480px) {
    margin-right: unset;
    margin-left: auto;
  }
`;
NavBar.Logo = styled.img`
  width: 75px;
  image-rendering: pixelated;
  filter: ${(props) => props.theme.logo.filter};
  @media (max-width: 480px) {
    margin-left: 0.5em;
    width: 40px;
  }
`;
NavBar.Menu = styled.div`
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  flex-grow: 1;
`;
NavBar.Link = styled(NavLink).attrs((props) => ({
  className: props.isActive ? "active" : "",
}))`
  padding: 1em;
  color: ${(props) => props.theme.colors.accent4};
  text-decoration: none;
  &:hover {
    color: ${(props) => props.theme.colors.text};
    background-color: ${(props) => props.theme.colors.accent1};
  }
  &.active {
    color: ${(props) => props.theme.colors.active};
  }
`;
NavBar.End = styled.div`
  margin-left: auto;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  > :not(:last-child) {
    @media (max-width: 480px) {
      margin-bottom: 0.4em;
    }
  }
`;
NavBar.Main = styled.div`
  display: flex;
  flex-wrap: wrap;
  @media (max-width: 480px) {
    display: none;
  }
`;
NavBar.Waitlist = styled.div`
  margin-right: 2em;
  @media (max-width: 480px) {
    margin-right: 0em;
    width: 100%;
  }
`;
NavBar.Name = styled.div`
  margin-right: 2em;
  @media (max-width: 480px) {
    margin-right: 0em;
    width: 100%;
  }
`;

export function Menu({ onChangeCharacter, onChangeWaitlist, theme, setTheme, sticker, setSticker }) {
  const whoami = React.useContext(AuthContext);
  const { t } = useTranslation();
  const [isOpenMobileView, setOpenMobileView] = React.useState(false);

  let loginBtn;
  if (whoami === false) {
    loginBtn = (
      <AButton
        title="Loading"
        variant="secondary"
        disabled
      >
        <FontAwesomeIcon icon={faSpinner} spinPulse />
      </AButton>
    );
  } else if (whoami === null) {
    loginBtn = (
      <NavButton end to="/auth/start" variant="primary">
        {t('login')}
      </NavButton>
    )
  } else {
    loginBtn = (
      <NavButton end to="/auth/logout" variant="secondary">
        {t("logout")}
      </NavButton>
    )
  }

  return (
    <NavBar>
      <NavBar.Header>
        <MobileNavButton isOpen={isOpenMobileView} setIsOpen={setOpenMobileView} />
        <NavBar.LogoLink to="/">
          <NavBar.Logo src={logoImage} alt="Winter Coalition" />
        </NavBar.LogoLink>
      </NavBar.Header>
      <NavBar.Menu>
        <NavBar.Main>
          <NavLinks whoami={whoami} />
        </NavBar.Main>
        <NavBar.End>
          {whoami && (
            <>
              <WaitlistContext.Consumer>
                {(waitlists) => waitlists && (
                  <NavBar.Waitlist>
                    <InputGroup fixed>
                      <Select
                        value={waitlists.active}
                        onChange={(evt) =>
                          onChangeWaitlist && onChangeWaitlist(parseInt(evt.target.value))
                        }
                        style={{ flexGrow: "1" }}
                      >
                        {waitlists.available.map((wl) => (
                          <option key={wl.id} value={wl.id}>
                            {wl.name} {wl.open ? '🟢' : '🔴' }
                          </option>
                        ))}
                      </Select>
                    </InputGroup>
                  </NavBar.Waitlist>
                )}
              </WaitlistContext.Consumer>

              <NavBar.Name>
                <InputGroup fixed>
                  <Select
                    value={whoami.current.id}
                    onChange={(evt) =>
                      onChangeCharacter && onChangeCharacter(parseInt(evt.target.value))
                    }
                    style={{ flexGrow: "1" }}
                  >
                    {whoami.characters.map((character) => (
                      <option key={character.id} value={character.id}>
                        {character.name}
                      </option>
                    ))}
                  </Select>
                  <NavButton end to="/auth/start">
                    <FontAwesomeIcon icon={faRotate} />
                  </NavButton>
                </InputGroup>
              </NavBar.Name>
            </>
          )}
          <InputGroup>
            <EventNotifier />
            <ThemeSelect
              theme={theme}
              setTheme={setTheme}
              sticker={sticker}
              setSticker={setSticker}
            />
            <AButton title="ReportBug" href="https://jira.winterco.org/projects/WAITLIST/issues" target="_blank">
              <FontAwesomeIcon icon={faBug} />
            </AButton>
            {loginBtn}
          </InputGroup>
        </NavBar.End>
        <MobileNav isOpen={isOpenMobileView} whoami={whoami} />
      </NavBar.Menu>
    </NavBar>
  );
}
