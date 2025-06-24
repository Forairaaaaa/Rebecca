import TemplateClock from "./components/template-clock";

export default function Home() {
  return (
    <main className="w-[100vw] h-[100vh] bg-amber-50 flex flex-col gap-24 place-items-center place-content-center">
      <div
        id="cover-screen-0"
        className="w-[280px] h-[240px] bg-black rounded-4xl"
      >
        <TemplateClock canvasId="cover-screen-0" />
      </div>
      <div
        id="cover-screen-1"
        className="w-[280px] h-[240px] bg-black rounded-4xl"
      >
        <TemplateClock canvasId="cover-screen-1" />
      </div>
    </main>
  );
}
