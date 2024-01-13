import { AppBar, Divider, Toolbar, Typography } from "@mui/material";
import { Outlet } from "react-router-dom";

export default function Root() {
  return (
    <>
      <AppBar position="static">
        <Toolbar>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Displex
          </Typography>
        </Toolbar>
      </AppBar>
      <Divider />
      <Outlet />
    </>
  );
}
