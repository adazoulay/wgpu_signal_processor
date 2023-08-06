import "./style.css";

document.querySelector("#app").innerHTML = `
  <div>
  <div>Canvas Below </div>
   <canvas />
  </div>
`;

setupCounter(document.querySelector("#counter"));
