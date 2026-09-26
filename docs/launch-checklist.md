# Launch checklist — one concentrated day

Generic steps and timing only. No paid promotion, no star exchanges — the story
has to earn every star it gets.

## The week before (30 minutes, once)

- Freeze the story: one sentence on what the repo is and why a visitor should
  star it; pin the demo GIF and the 0-0-0 baseline scoreboard as the hook
  ("everything ties at zero — beat the dummies")
- Re-run the full gate set locally: `cargo build --release`,
  `cargo clippy --all-targets`, `cargo test` — launch day is the wrong day to
  discover a red build
- Confirm CI on `main` is green after the package push

## Upload the social preview (manual, 5 minutes)

- Open the repository on GitHub, then Settings -> General -> "Social preview"
- Click Edit -> Upload an image -> choose `docs/social-preview.png`
  (PNG, 1280 x 640, under 1 MB — exactly what GitHub asks for)
- Save, then re-scrape: social platforms cache previews, so re-post or refresh
  any link after uploading to avoid the blank-gray card

## Launch day (the timing plan)

- Day-of timing skeleton: morning (T-0, local 09:00-ish) post the main link
  once, with the demo GIF inline and the one-sentence story; first impressions
  in the first hour matter most for trending
- Mid-morning (+2h): answer every comment and question that arrived; reply
  threads keep the repo climbing
- Afternoon (+6h): second wave — share the same story in a different wording
  for people the morning post missed
- Evening (+10h): recap post ("day one: N stars, top pull request so far: ...")
  to catch the late crowd and close the day

## The day after

- Triage issues and PRs from the wave; a same-day merge of an outside bot is
  the best possible follow-up story
- Thank contributors by name in the recap
- File what to improve next (scoring story, more robots) as issues, not
  promises
