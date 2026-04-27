import { useState } from "react";
import type { PlanKind } from "../types";

interface FirstRunWizardProps {
  onDone: (plan: PlanKind, autoStart: boolean) => void;
}

type Step = 1 | 2 | 3;

const PLAN_OPTIONS: { value: PlanKind; label: string }[] = [
  { value: "pro", label: "Pro — 44k tokens / Block" },
  { value: "max5", label: "Max 5× — 88k tokens / Block" },
  { value: "max20", label: "Max 20× — 220k tokens / Block" },
];

export function FirstRunWizard({ onDone }: FirstRunWizardProps) {
  const [step, setStep] = useState<Step>(1);
  const [selectedPlan, setSelectedPlan] = useState<PlanKind>("max5");
  const [autoStart, setAutoStart] = useState(true);

  const next = () => setStep((s) => (s < 3 ? ((s + 1) as Step) : s));

  const finish = () => onDone(selectedPlan, autoStart);

  return (
    <div style={styles.overlay}>
      <div style={styles.card}>
        {/* Step indicator */}
        <div style={styles.steps}>
          {([1, 2, 3] as Step[]).map((s) => (
            <div
              key={s}
              style={{
                ...styles.dot,
                backgroundColor: s <= step ? "var(--accent)" : "var(--border-subtle)",
              }}
            />
          ))}
        </div>

        {step === 1 && (
          <>
            <div style={styles.emoji}>🔥</div>
            <h2 style={styles.heading}>Willkommen bei Ignis</h2>
            <p style={styles.body}>
              Ignis zeigt dir Token-Verbrauch, Kosten und aktive Sessions
              deiner Claude&nbsp;Code-Nutzung — lokal, ohne Cloud.
            </p>
            <p style={styles.body}>
              Daten kommen aus{" "}
              <code style={styles.code}>%USERPROFILE%\.claude\projects\</code>
              {" "}— read-only, nichts wird hochgeladen.
            </p>
            <button style={styles.btnPrimary} onClick={next}>
              Los geht's →
            </button>
          </>
        )}

        {step === 2 && (
          <>
            <h2 style={styles.heading}>Welchen Plan nutzt du?</h2>
            <p style={styles.body}>
              Das Token-Limit pro 5-Stunden-Block bestimmt, ab wann Ignis dich warnt.
            </p>
            <div style={styles.planList}>
              {PLAN_OPTIONS.map((opt) => (
                <label key={opt.value} style={styles.planRow}>
                  <input
                    type="radio"
                    name="plan"
                    value={opt.value}
                    checked={selectedPlan === opt.value}
                    onChange={() => setSelectedPlan(opt.value)}
                    style={styles.radio}
                  />
                  <span style={styles.planLabel}>{opt.label}</span>
                </label>
              ))}
            </div>
            <button style={styles.btnPrimary} onClick={next}>
              Weiter →
            </button>
          </>
        )}

        {step === 3 && (
          <>
            <h2 style={styles.heading}>Auto-Start</h2>
            <p style={styles.body}>
              Soll Ignis automatisch mit Windows starten?
              Du kannst das später in den Einstellungen ändern.
            </p>
            <label style={styles.checkRow}>
              <input
                type="checkbox"
                checked={autoStart}
                onChange={(e) => setAutoStart(e.target.checked)}
                style={styles.checkbox}
              />
              <span style={styles.checkLabel}>Auto-Start aktivieren</span>
            </label>
            <button style={styles.btnPrimary} onClick={finish}>
              Abschließen ✓
            </button>
          </>
        )}
      </div>
    </div>
  );
}

const styles = {
  overlay: {
    position: "absolute" as const,
    inset: 0,
    backgroundColor: "var(--bg-base)",
    zIndex: 20,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "16px",
  },
  card: {
    width: "100%",
    display: "flex",
    flexDirection: "column" as const,
    gap: "12px",
    alignItems: "flex-start",
  },
  steps: {
    display: "flex",
    gap: "6px",
    marginBottom: "4px",
  },
  dot: {
    width: "6px",
    height: "6px",
    borderRadius: "50%",
    transition: "background-color 200ms",
  },
  emoji: {
    fontSize: "32px",
    lineHeight: 1,
  },
  heading: {
    fontSize: "18px",
    fontWeight: 600,
    color: "var(--text-primary)",
    margin: 0,
  },
  body: {
    fontSize: "12px",
    color: "var(--text-secondary)",
    lineHeight: 1.5,
    margin: 0,
  },
  code: {
    fontFamily: "var(--font-mono)",
    fontSize: "10px",
    color: "var(--accent)",
    backgroundColor: "var(--bg-elevated)",
    padding: "1px 4px",
    borderRadius: "3px",
  },
  planList: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "8px",
    width: "100%",
  },
  planRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    cursor: "pointer",
  },
  radio: {
    accentColor: "var(--accent)",
    width: "14px",
    height: "14px",
    cursor: "pointer",
  },
  planLabel: {
    fontSize: "13px",
    color: "var(--text-secondary)",
  },
  checkRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    cursor: "pointer",
  },
  checkbox: {
    accentColor: "var(--accent)",
    width: "14px",
    height: "14px",
    cursor: "pointer",
  },
  checkLabel: {
    fontSize: "13px",
    color: "var(--text-secondary)",
  },
  btnPrimary: {
    marginTop: "4px",
    padding: "8px 20px",
    backgroundColor: "var(--accent)",
    color: "#fff",
    border: "none",
    borderRadius: "6px",
    fontSize: "13px",
    fontWeight: 600,
    cursor: "pointer",
    fontFamily: "var(--font-sans)",
    alignSelf: "flex-end",
  },
} as const;
