import { Button, Grid } from "@mui/material";

export default function Recap() {
  return (
      <Grid container spacing={2}>
        <Grid item xs={4}>
          <Button>Signin with Discord</Button>
        </Grid>
      </Grid>
  );
}
