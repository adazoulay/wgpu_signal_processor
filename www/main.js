import "./style.css";

import init from "wasm_wgpu_template";

init().then((instance) => {
  console.log("it worked");
  //   instance.exports.test();
});

document.querySelector("#app").innerHTML = `
  <div>
  <div>Canvas Below </div>
   <canvas id="wasm-example" />
  </div>
`;

setupCounter(document.querySelector("#counter"));
