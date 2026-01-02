import React from "react";
import { Badge } from "../../Components/Badge";

export class AltNumberDisplay extends React.Component {
  constructor(props) {
    super(props);
    this.state = { };
  }

  render() {
    const altNumber = this.props.altNumber ?? 0;
    if (altNumber > 0) {
      return (
        <span title={altNumber + " alt(s) exists in the fleet"}>
          <Badge variant="warning">{altNumber}x</Badge>
        </span>
      );
    } else {
      return <Badge variant="secondary">New</Badge>;
    }
  }
}
