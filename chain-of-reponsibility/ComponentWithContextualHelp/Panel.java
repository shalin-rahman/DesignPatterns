/**
 * Panel - Concrete Intermediate Handler (layout container).
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI / Help-System example.
 *
 * <h2>Role in Chain of Responsibility Pattern</h2>
 * <ul>
 *   <li>A mid-level container that groups several {@link Dialog}s or other components.</li>
 *   <li>Receives the help request when neither a child Button nor its parent Dialog
 *       could handle it.</li>
 *   <li>If the Panel has a contextual help topic configured, it shows a pop-up and
 *       terminates the chain.  Otherwise, the request propagates to the top-level
 *       {@link Window}.</li>
 * </ul>
 *
 * <h2>Chain position</h2>
 * <pre>
 *   Button → Dialog → [Panel] → Window
 * </pre>
 *
 * <h2>Pattern insight — why CoR, not Composite?</h2>
 * A {@link Panel} in a Composite pattern would call {@code showHelp()} on <em>all</em>
 * of its children (top-down broadcast).  In CoR, the Panel only acts when the request
 * <em>reaches</em> it from below, and it acts on <em>its own</em> content — not on
 * its children.  Direction and participation are the key differences.
 */
public class Panel extends Container {

    /**
     * A generic help topic or pop-up text this panel can display.
     * If non-null, this Panel is a terminating handler.
     */
    private final String modalHelpText;

    /**
     * Constructs a Panel without a help topic — requests pass through to the Window.
     */
    public Panel() {
        this.modalHelpText = null;
    }

    /**
     * Constructs a Panel with a modal help text, making it able to terminate the chain.
     *
     * @param modalHelpText the help content to display in a pop-up modal dialog.
     */
    public Panel(String modalHelpText) {
        this.modalHelpText = modalHelpText;
    }

    /**
     * Handles the help request by showing a modal help pop-up if configured;
     * otherwise, forwards to the enclosing Window.
     */
    @Override
    public void showHelp() {
        if (modalHelpText != null) {
            // Panel has a generic help topic — chain stops here.
            System.out.println("[Panel] Showing modal help pop-up: \"" + modalHelpText + "\"");
        } else {
            System.out.println("[Panel] No modal help — forwarding to " +
                    (container != null ? container.getClass().getSimpleName() : "nobody (end of chain)") + "...");
            if (container != null) {
                container.showHelp();
            } else {
                System.out.println("[Panel] End of chain — no help available.");
            }
        }
    }

    /**
     * Returns the modal help text, or {@code null} if unset.
     *
     * @return modal help text.
     */
    public String getModalHelpText() {
        return modalHelpText;
    }
}
