import "./App.css";
import Logger from "./common/Logger";

function App() {
  async function onLogClick() {
    await Logger.info("word_highlight_assistant", {
      message: "这是一条日志",
      code: 200,
    });
  }

  return (
    <main className="container">
      <div>
        <button onClick={onLogClick}>打印日志</button>
      </div>
    </main>
  );
}

export default App;
