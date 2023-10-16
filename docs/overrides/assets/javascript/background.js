const background = document.getElementById("background");
if (background !== undefined) {
  let ctx = background?.getContext("2d");

  let columns = [];

  function updateMatrix() {
    if (ctx && background) {
      const w = (background.width = document.body.offsetWidth);
      const h = (background.height = document.body.offsetHeight);

      columns = Array.from(
        { length: Math.floor(w / 12) + 1 },
        () => Math.floor((Math.random() * h) / 12) * 12
      );
    }
  }

  function renderCanvas() {
    if (ctx && background) {
      ctx.fillStyle = "#0001";
      ctx.fillRect(0, 0, background.width, background.height);

      ctx.fillStyle = "#222";
      ctx.font = "1em monospace";

      columns.forEach((y, ind) => {
        const text = String.fromCharCode(Math.floor(Math.random() * 96 + 32));
        const x = ind * 12;
        ctx?.fillText(text, x, y);
        if (y > 100 + Math.random() * 10000) {
          columns[ind] = 0;
        } else {
          columns[ind] = y + 12;
        }
      });
    }
  }

  setInterval(() => {
    window.requestAnimationFrame(renderCanvas);
  }, 50);
  updateMatrix();
  window.addEventListener("resize", updateMatrix);
}
