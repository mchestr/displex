import { gql, useQuery } from "@apollo/client";
import Loader from "../../common/Loader/index.tsx";
import RecapHero from "../../components/RecapHero.tsx";
import TotalCount from "../../components/TotalCount.tsx";
import LeaderBoard from "../../components/LeaderBoard.tsx";
import SignIn from "../Authentication/SignIn.tsx";

const WHOAMI_QUERY = gql`
query WhoAmI {
  whoami {
    discordUserId
    plexUserId
    role
  }
}`

const GET_RECAP = gql`
query RecapQuery {
  topMedia {
    ... on TopMedia {
      topMovie
      topShow
    }
  }
  leaderboard {
    ... on Leaderboard {
      watchPosition
      watchCount
      watchDuration
    }
  }
}
`;

const Recap = () => {
  const { loading, error, data } = useQuery(GET_RECAP);
  return loading || error ? (
    <Loader />
  ) : (
    <>
      <RecapHero totalMinutes={data.leaderboard.watchDuration} totalWatchCount={data.leaderboard.watchCount} />
      <div className="grid h-16 place-items-center mt-4">
        <div className="content-center">
          <div className="grid grid-cols-2 gap-4 mt-4 md:grid-cols-2 md:gap-6 xl:grid-cols-2 2xl:gap-7.5">
            <LeaderBoard position={data.leaderboard.watchPosition} basedOn="Based on total minutes watched" />
            <TotalCount stat={data.leaderboard.watchCount} statName="Total play count"/>
          </div>
          <div className="grid grid-cols-1 gap-4 mt-4 md:grid-cols-2 md:gap-6 xl:grid-cols-2 2xl:gap-7.5">
            <TotalCount stat={data.topMedia.topMovie} statName="Top Movie Watched"/>
            <TotalCount stat={data.topMedia.topShow} statName="Top Show Watched"/>
          </div>
        </div>
      </div>
    </>
  );
};

const EntryPoint = () => {
  const { loading, error, data } = useQuery(WHOAMI_QUERY);

  return loading || error ? (
    <Loader />
  ) : (
    data.whoami.plexUserId ? <Recap /> : <SignIn />
  )
}

export default EntryPoint;
