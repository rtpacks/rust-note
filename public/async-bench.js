const request = () => fetch("http://127.0.0.1:7878").then((res) => res.text());
// .then((res) => console.log(res));

const main = async () => {
  const start = new Date();
  const requests = new Array(2000)
    .fill(async () => {
      // console.log(`start ${index}: Iteration`);
      await request();
    })
    .map((item) => item());

  await Promise.all(requests);
  const end = new Date();

  console.log(`cost: ${end - start}`);
};

main();
