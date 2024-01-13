import {
  Alert,
  Card,
  CardContent,
  Container,
  Drawer,
  Grid,
  IconButton,
  Paper,
  Skeleton,
  Switch,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Typography,
} from "@mui/material";
import { gql, useQuery } from "@apollo/client";
import Moment from "moment";
import { Link, Outlet } from "react-router-dom";
import React from "react";
import CloseIcon from "@mui/icons-material/Close";

const ADMIN_LOAD_QUERY = gql`
  query AdminLoad {
    listDiscordUsers {
      id
      username
      createdAt
      updatedAt
      isActive
    }
    getPlexStatus {
      ... on ServerStatus {
        connected
      }
    }
    getPlexActivity {
      ... on GetActivity {
        streamCount
      }
    }
  }
`;

function Admin() {
  const { loading, error, data } = useQuery(ADMIN_LOAD_QUERY);
  const [state, setState] = React.useState({
    bottom: false,
  });
  const toggleDrawer =
    (anchor: string, open: boolean) =>
    (event: React.KeyboardEvent | React.MouseEvent) => {
      if (
        event.type === "keydown" &&
        ((event as React.KeyboardEvent).key === "Tab" ||
          (event as React.KeyboardEvent).key === "Shift")
      ) {
        return;
      }

      setState({ ...state, [anchor]: open });
    };
  return (
    <>
      <Container style={{ marginTop: "10px" }}>
        <Grid container spacing={2}>
          <Grid item xs={4}>
            <Card sx={{ minWidth: 275 }}>
              <CardContent>
                <Typography
                  sx={{ fontSize: 14 }}
                  color="text.secondary"
                  gutterBottom
                >
                  Plex Status
                </Typography>
                <Typography variant="h5" component="div">
                  {error || loading ? (
                    <Skeleton variant="text" height={10} width="40%" />
                  ) : data.getPlexStatus.connected ? (
                    "Online"
                  ) : (
                    "Offline"
                  )}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={4}>
            <Card sx={{ minWidth: 275 }}>
              <CardContent>
                <Typography
                  sx={{ fontSize: 14 }}
                  color="text.secondary"
                  gutterBottom
                >
                  Total Subscribers
                </Typography>
                <Typography variant="h5" component="div">
                  {error || loading ? (
                    <Skeleton variant="text" height={10} width="40%" />
                  ) : (
                    data.listDiscordUsers.length
                  )}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={4}>
            <Card sx={{ minWidth: 275 }}>
              <CardContent>
                <Typography
                  sx={{ fontSize: 14 }}
                  color="text.secondary"
                  gutterBottom
                >
                  Current Streams
                </Typography>
                <Typography variant="h5" component="div">
                  {error || loading ? (
                    <Skeleton variant="text" height={10} width="40%" />
                  ) : (
                    data.getPlexActivity.streamCount
                  )}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12}>
            {error ? (
              <Alert severity="error">{error.message}</Alert>
            ) : loading ? (
              <Skeleton variant="rectangular" />
            ) : (
              <TableContainer component={Paper}>
                <Table sx={{ minWidth: 650 }} aria-label="simple table">
                  <TableHead>
                    <TableRow>
                      <TableCell>Username</TableCell>
                      <TableCell>Created</TableCell>
                      <TableCell>Last Updated</TableCell>
                      <TableCell>Active</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {data.listDiscordUsers.map((row: any) => (
                      <TableRow
                        key={row.id}
                        sx={{
                          "&:last-child td, &:last-child th": { border: 0 },
                        }}
                      >
                        <TableCell>
                          <Link
                            to={`/admin/users/${row.id}`}
                            onClick={toggleDrawer("bottom", true)}
                            style={{ textDecoration: 'none' }}
                          >
                            <Typography variant="h6">{row.username}</Typography>
                          </Link>
                        </TableCell>
                        <TableCell>
                          {Moment(row.createdAt).format("lll")}
                        </TableCell>
                        <TableCell>
                          {Moment(row.updatedAt).format("lll")}
                        </TableCell>
                        <TableCell>
                          <Switch checked={row.isActive} />
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            )}
          </Grid>
        </Grid>
      </Container>
      <Drawer
        anchor="bottom"
        open={state.bottom}
        onClose={toggleDrawer("bottom", false)}
      >
        <IconButton
          onClick={toggleDrawer("bottom", false)}
        >
          <CloseIcon style={{ position: "absolute" }} />
        </IconButton>
        <Outlet />
      </Drawer>
    </>
  );
}

export default Admin;
