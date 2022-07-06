// import ReactMarkdown from "react-markdown";
import React from 'react';
import remarkGfm from "remark-gfm";
import { NavLink } from "react-router-dom";
import { Table, TableHead, TableBody, Cell, CellHead, Row } from "./Table";

const ReactMarkdown = React.lazy(() => import("react-markdown"));

function Link({ href, children, ...props }) {
  return href.startsWith("/") ? (
    <NavLink end to={href}>
      {children}
    </NavLink>
  ) : (
    <a href={href} {...props}>
      {children}
    </a>
  );
}

const components = {
  a: Link,
  table: Table,
  thead: TableHead,
  tbody: TableBody,
  th: CellHead,
  td: Cell,
  tr: Row,
};

export function Markdown({ ...args }) {
  return <ReactMarkdown components={components} remarkPlugins={[remarkGfm]} {...args} />;
}
