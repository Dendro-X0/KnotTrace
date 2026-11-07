# Frontend spec — Protect review pass

Status: **shipped in tree**  
Builds on: Protect trust hero · auto-protect log · Assist apply paths

## Goal

One click on Protect processes the **to review** list honestly: apply safe reversible assists, mark indicator-only rows reviewed for this session.

## Placement

Inside the trust / caution card, above or below the alert list:

```text
[ Use caution · N to review ]
[ … alert rows … ]
[ Process review ]   ← primary when N>0
[ Applied DNS · 3 indicators marked reviewed ]
```

## Copy

| Key | en | zh-Hans |
|-----|----|---------|
| `protect.processReview` | Process review | 一键处理 |
| `protect.processReviewBusy` | Processing… | 处理中… |
| `protect.processReviewHint` | Applies reversible DNS/Connect when available; marks the rest reviewed (indicators are not auto-fixed). | 有可用时应用可逆的 DNS/连接辅助；其余标为已审阅（指标不会被自动“修复”）。 |

## States

- Idle + alerts → button enabled  
- Running → busy label, disabled  
- Done → summary string; acked rows leave the active list (or show “Reviewed” if we keep them — **prefer hide from active queue**)  
- No assists possible → still acknowledge indicators; note explains  

## Non-goals

- Closing OS listeners / killing processes  
- Fake “optimized” Mbps  
- Silent auto without the button  
