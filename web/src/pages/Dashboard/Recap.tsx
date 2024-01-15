import { gql, useQuery } from "@apollo/client";
import Loader from "../../common/Loader/index.tsx";
import RecapHero from "../../components/RecapHero.tsx";
import TotalCount from "../../components/TotalCount.tsx";
import LeaderBoard from "../../components/LeaderBoard.tsx";

const GET_RECAP = gql`
  query RecapQuery {
    whoami {
      discordUserId
      plexUserId
      role
    }
    watchStats {
      showCount
      movieCount
      totalMinutes
    }
    leaderboard {
      requestPosition
      watchPosition
    }
  }
`;

const MOCK_DATA = {
  data: {
    whoami: {
      discordUserId: "1",
      plexUserId: "2",
      role: "3",
    },
    watchStats: {
      showCount: 10,
      movieCount: 100,
      totalMinutes: 1000,
    },
    leaderboard: {
      requestPosition: 3,
      watchPosition: 5,
    }
  },
  loading: false,
  error:false,
};

const Recap = () => {
  // const { loading, error, data } = useQuery(GET_RECAP);
  const { loading, error, data } = MOCK_DATA;
  return loading || error ? (
    <Loader />
  ) : (
    <>
      <RecapHero totalMinutes={data.watchStats.totalMinutes} />
      <div className="grid h-16 place-items-center mt-4">
        <div className="content-center">
          <div className="grid grid-cols-2 gap-4 mt-4 md:grid-cols-2 md:gap-6 xl:grid-cols-2 2xl:gap-7.5">
            <LeaderBoard position={data.leaderboard.watchPosition} basedOn="Based on minutes watched" />
            <LeaderBoard position={data.leaderboard.requestPosition} basedOn="Based on request count" />
          </div>
          <div className="grid grid-cols-1 gap-4 mt-4 md:grid-cols-2 md:gap-6 xl:grid-cols-3 2xl:gap-7.5">
            <TotalCount count={data.watchStats.totalMinutes} mediaType="Watched Minutes"/>
            <TotalCount count={data.watchStats.showCount} mediaType="Shows"/>
            <TotalCount count={data.watchStats.movieCount} mediaType="Movies"/>
          </div>
        </div>
      </div>
    </>
  );
};

export default Recap;
