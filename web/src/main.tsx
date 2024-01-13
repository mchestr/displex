import React from "react";
import ReactDOM from "react-dom/client";
import "./index.css";

import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";

import { ApolloClient, InMemoryCache, ApolloProvider } from "@apollo/client";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

import { createBrowserRouter, RouterProvider } from "react-router-dom";
import Root from "./routes/root";
import ErrorPage from "./error-page";
import Admin from "./routes/admin";
import UserSummary from "./routes/user-summary";
import Recap from "./routes/recap";

const darkTheme = createTheme({
  palette: {
    mode: "dark",
  },
});

const client = new ApolloClient({
  uri: "http://localhost:8080/gql",
  cache: new InMemoryCache(),
});

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
    errorElement: <ErrorPage />,
    children: [
      {
        path: "/admin",
        element: <Admin />,
        children: [
          {
            path: "users/:id",
            element: <UserSummary />,
          },
        ],
      },
      {
        path: "/recap/:year",
        element: <Recap />,
      },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider theme={darkTheme}>
      <ApolloProvider client={client}>
        <CssBaseline />
        <RouterProvider router={router} />
      </ApolloProvider>
    </ThemeProvider>
  </React.StrictMode>
);
