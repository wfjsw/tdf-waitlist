import React, { Suspense } from "react";
import { BrowserRouter as Router } from "react-router-dom";
import * as Sentry from "@sentry/react";

import { processAuth } from "../Pages/Auth";
import { ToastDisplay } from "../Components/Toast";
import { AuthContext, ToastContext, EventContext, WaitlistContext } from "../contexts";
import { ThemeProvider, createGlobalStyle } from "styled-components";
import { Routes } from "./routes";
import { Container } from "react-awesome-styled-grid";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPowerOff } from "@fortawesome/pro-solid-svg-icons";

import { Menu } from "./Menu";
import "./reset.css";
import theme from "./theme.js";
import { processWaitlists } from "../waitlists";
// import { useQuery } from "../Util/query";

const GlobalStyle = createGlobalStyle`
  html {
    overflow-y: scroll;
    text-rendering: optimizeLegibility;
    font-size: 16px;
    min-width: 300px;
  }
  body {
    min-height: 100vh;
    background-color: ${(props) => props.theme.colors.background};
    color: ${(props) => props.theme.colors.text};
    font-family: ${(props) => props.theme.font.family};
    line-height: 1.5;
    font-weight: 400;
	${(props) =>
    props.theme.sticker &&
    `
	  &:before {
	   content:'';
	   pointer-events:none;
	   position:fixed;
	   z-index:9001;
	   width:100%;
	   height:100%;
	   background-position:100% 100%;
	   background-repeat: no-repeat;
	   opacity:1;
	   background-size: 18%;
       background-image: url(${props.theme.sticker});
    }
  `}
  }
  em, i {
    font-style: italic;
  }
  strong, b {
    font-weight: bold;
  }
`;

function Spinner() {
  return (<div><i>Downloading extra components...</i></div>);
}

function ErrorDisplay({resetError}) {
  return (
    <>
      <div>An error has occurred. Please click the following button to reload.</div>
      <button onClick={() => resetError()}><FontAwesomeIcon icon={faPowerOff} /></button>
    </>
  );
}

export default class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      auth: false,
      waitlists: null,
      toasts: [],
      events: null,
      eventErrors: 0,
      theme:
        (window.localStorage &&
          window.localStorage.getItem("theme") in theme &&
          window.localStorage.getItem("theme")) ||
        "Light",
    };
  }

  componentDidUpdate() {
    if (this.state.auth && !this.state.events) {
      var events = new EventSource("/api/sse/stream");
      events.addEventListener("open", (evt) => {
        this.setState({ eventErrors: 0 });
      });
      events.addEventListener("error", (err) => {
        events.close();
        setTimeout(() => {
          this.setState({ events: null, eventErrors: this.state.eventErrors + 1 });
        }, this.state.eventErrors * 5000 + Math.random() * 10000);
      });

      events.addEventListener("waitlist_update", async () => {
        this.updateWaitlistOverview();
      });
      this.setState({ events });
    }
  }

  componentDidMount() {
    processAuth((whoami) => this.setState({ auth: whoami }));
    this.updateWaitlistOverview();
  }

  updateWaitlistOverview() {
    if (this.state.waitlists === null) {
      const params = new URLSearchParams(window.location.search);
      const qwl = params.has("wl") && !isNaN(params.get("wl")) ? parseInt(params.get("wl")) : null;
      const prevWl = window.localStorage && parseInt(window.localStorage.getItem("activewl"));
      const rwl = qwl ?? prevWl;
      processWaitlists((wls) => {
        this.setState({
          waitlists: { available: wls, active: wls.some(n => n.id === rwl) ? rwl : wls[0].id }
        })
        // params.set("wl", this.state.waitlists.active);
        // window.history.replaceState({}, "", `${window.location.pathname}?${params.toString()}`);
      });
    } else {
      let active = this.state.waitlists.active
      processWaitlists((wls) => this.setState({ waitlists: { available: wls, active } }));
    }
  }

  addToast = (toast) => {
    this.setState({ toasts: [...this.state.toasts, toast] });
  };

  changeCharacter = (newChar) => {
    var newState = { ...this.state.auth };
    var theChar = this.state.auth.characters.filter((char) => char.id === newChar)[0];
    newState.current = theChar;
    this.setState({ auth: newState });
  };

  changeWaitlist = (newWlID) => {
    // const params = new URLSearchParams(window.location.search);
    let newState = { ...this.state.waitlists };
    newState.active = newWlID;
    window.localStorage.setItem("activewl", newWlID);
    // params.set("wl", newWlID);
    // window.history.replaceState({}, "", `${window.location.pathname}?${params.toString()}`);
    this.setState({
      waitlists: newState
    })
  }

  render() {
    return (
        <ThemeProvider theme={theme[this.state.theme]}>
          <GlobalStyle />
          <ToastContext.Provider value={this.addToast}>
          <EventContext.Provider value={this.state.events}>
            <AuthContext.Provider value={this.state.auth}>
              <WaitlistContext.Provider value={this.state.waitlists}>
                <Router>
                  <Container>
                    <Menu
                      onChangeCharacter={(char) => this.changeCharacter(char)}
                      onChangeWaitlist={(waitlist) => this.changeWaitlist(waitlist)}
                      theme={this.state.theme}
                      setTheme={(newTheme) => {
                        this.setState({ theme: newTheme });
                        if (window.localStorage) {
                          window.localStorage.setItem("theme", newTheme);
                        }
                      }}
                    />
                    <Sentry.ErrorBoundary fallback={<ErrorDisplay />} showDialog>
                      <Suspense fallback={<Spinner />}>
                        <Routes />
                      </Suspense>
                    </Sentry.ErrorBoundary>
                    <ToastDisplay
                      toasts={this.state.toasts}
                      setToasts={(toasts) => this.setState({ toasts })}
                    />
                  </Container>
                </Router>
              </WaitlistContext.Provider>
            </AuthContext.Provider>
          </EventContext.Provider>
          </ToastContext.Provider>
        </ThemeProvider>
    );
  }
}
