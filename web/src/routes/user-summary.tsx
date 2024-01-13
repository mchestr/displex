import { gql, useQuery } from "@apollo/client";
import {
  Card,
  CardContent,
  CardHeader,
  Chip,
  Grid,
  Paper,
  Skeleton,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Tooltip,
  Typography,
} from "@mui/material";
import { useParams } from "react-router-dom";
import Moment from "moment";
import React from "react";

const USER_SUMMARY_QUERY = gql`
  query UserSummary($id: String!) {
    userSummary(input: { id: $id }) {
      ... on SummaryDiscordUserSuccess {
        summary {
          discordUser {
            id
            username
            createdAt
            updatedAt
            isActive
          }
          plexUsers {
            id
            username
            createdAt
            updatedAt
            isSubscriber
          }
          plexTokens {
            accessToken
            createdAt
          }
          discordTokens {
            accessToken
            createdAt
            expiresAt
            status
          }
        }
      }
    }
  }
`;

const UserSummary: React.FC = () => {
  const { id } = useParams();
  const { loading, error, data } = useQuery(USER_SUMMARY_QUERY, {
    variables: {
      id,
    },
  });
  return (
    <Grid container spacing={2} style={{ padding: "2%" }}>
      <Grid item xs={6}>
        <Card sx={{ minWidth: 275 }}>
          <CardContent>
            <Typography variant="h5" color="text.secondary" gutterBottom>
              <b>Discord User</b>
            </Typography>
            <Typography variant="h5" component="div">
              {loading || error ? (
                <Skeleton variant="text" />
              ) : (
                <Card>
                  <CardHeader
                    title={
                      <Typography variant="h5">
                        {data.userSummary.summary.discordUser.username}{" "}
                        {data.userSummary.summary.discordUser.isActive ? (
                          <Chip
                            label="active"
                            color="success"
                            variant="outlined"
                          />
                        ) : (
                          <Chip
                            label="inactive"
                            color="error"
                            variant="outlined"
                          />
                        )}
                      </Typography>
                    }
                    subheader={
                      <Tooltip
                        title={Moment(
                          data.userSummary.summary.discordUser.createdAt
                        ).format("lll")}
                      >
                        <Typography>
                          {Moment(
                            data.userSummary.summary.discordUser.createdAt
                          ).fromNow()}
                        </Typography>
                      </Tooltip>
                    }
                  />
                </Card>
              )}
            </Typography>
          </CardContent>
        </Card>
      </Grid>
      <Grid item xs={12}>
        <Typography
          sx={{ flex: "1 1 100%" }}
          variant="h6"
          id="tableTitle"
          component="div"
        >
          Plex Users
        </Typography>
        <TableContainer component={Paper}>
          <Table sx={{ minWidth: 650 }} aria-label="simple table">
            <TableHead>
              <TableRow>
                <TableCell>Username</TableCell>
                <TableCell>Created</TableCell>
                <TableCell>Updated</TableCell>
                <TableCell>Subscriber</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {!loading &&
                data.userSummary.summary.plexUsers.map((row) => (
                  <TableRow key={row.id}>
                    <TableCell>{row.username}</TableCell>
                    <TableCell>{Moment(row.createdAt).format("lll")}</TableCell>
                    <TableCell>{Moment(row.updatedAt).format("lll")}</TableCell>
                    <TableCell>
                      <Chip
                        label={row.isSubscriber ? "true" : "false"}
                        color={row.isSubscriber ? "success" : "error"}
                        variant="outlined"
                      />
                    </TableCell>
                  </TableRow>
                ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Grid>
      <Grid item xs={6}>
        <Typography
          sx={{ flex: "1 1 100%" }}
          variant="h6"
          id="tableTitle"
          component="div"
        >
          Discord Tokens
        </Typography>
        <TableContainer component={Paper}>
          <Table sx={{ minWidth: 650 }} aria-label="simple table">
            <TableHead>
              <TableRow>
                <TableCell>Access Token</TableCell>
                <TableCell>Expires</TableCell>
                <TableCell>Created</TableCell>
                <TableCell>Status</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {!loading &&
                data.userSummary.summary.discordTokens.map((row) => (
                  <TableRow key={row.accessToken}>
                    <TableCell>{row.accessToken}</TableCell>
                    <TableCell>
                      <Tooltip title={Moment(row.expiresAt).format("lll")}>
                        <Typography>{Moment(row.expiresAt).fromNow()}</Typography>
                      </Tooltip>
                    </TableCell>
                    <TableCell>{Moment(row.createdAt).format("lll")}</TableCell>
                    <TableCell>
                      <Chip
                        label={row.status.toLowerCase()}
                        color={row.status == "ACTIVE" ? "success" : "error"}
                        variant="outlined"
                      />
                    </TableCell>
                  </TableRow>
                ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Grid>
      <Grid item xs={6}>
        <Typography
          sx={{ flex: "1 1 100%" }}
          variant="h6"
          id="tableTitle"
          component="div"
        >
          Plex Tokens
        </Typography>
        <TableContainer component={Paper}>
          <Table sx={{ minWidth: 650 }} aria-label="simple table">
            <TableHead>
              <TableRow>
                <TableCell>Access Token</TableCell>
                <TableCell>Created</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {!loading &&
                data.userSummary.summary.plexTokens.map((row) => (
                  <TableRow key={row.accessToken}>
                    <TableCell>{row.accessToken}</TableCell>
                    <TableCell>{Moment(row.createdAt).format("lll")}</TableCell>
                  </TableRow>
                ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Grid>
    </Grid>
  );
}

export default UserSummary;
