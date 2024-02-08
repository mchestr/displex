const RecapHero = (props: {
  totalMinutes: number,
  totalWatchCount: number,
}) => {
  var phrase = "That's not a lot of minutes, oh well, not everyone has time to binge the new season of Traitors."
  if (props.totalMinutes >= 100 && props.totalMinutes < 1000) {
    phrase = "Wowzer, you could have run a marathon in that time";
  } else if (props.totalMinutes >= 1000 && props.totalMinutes < 10000) {
    phrase = "That's a lot of media you binged..."
  } else if (props.totalMinutes >= 10000) {
    phrase = "You could have become an expert in something in that amount of time, but that would have been a lot harder."
  }
  return (
    <section className="bg-white dark:bg-gray-900">
      <div className="py-8 px-4 mx-auto max-w-screen-xl text-center lg:py-16 lg:px-12">
        <h1 className="mb-4 text-4xl font-extrabold tracking-tight leading-none text-gray-900 md:text-5xl lg:text-6xl dark:text-white">
          MFlix Stats
        </h1>
        <p className="mb-2 text-lg font-normal text-gray-500 lg:text-xl sm:px-16 xl:px-48 dark:text-gray-400">
          You've watched an amazing <b>{props.totalMinutes}</b> minutes! {phrase}
        </p>
      </div>
    </section>
  );
};

export default RecapHero;
