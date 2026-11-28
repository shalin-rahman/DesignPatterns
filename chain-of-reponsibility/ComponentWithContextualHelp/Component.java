/**
 * Base abstract handler holding a successor reference and default pass-along logic.
 */
public abstract class Component implements ComponentWithContextualHelp {

    /** Optional tooltip text displayed if this component can handle the request. */
    private String tooltipText;

    /** Successor link pointing to the enclosing UI container in the chain. */
    protected Component container;

    /**
     * Sets the next handler (enclosing container) in the chain.
     */
    public void setContainer(Component container) {
        this.container = container;
    }

    /**
     * Handles the request if a tooltip is set; otherwise forwards to the successor container.
     */
    @Override
    public void showHelp() {
        if (tooltipText != null) {
            System.out.println("[" + getClass().getSimpleName() + "] Showing tooltip: \"" + tooltipText + "\"");
        } else if (container != null) {
            System.out.println("[" + getClass().getSimpleName() + "] No tooltip — forwarding to " + container.getClass().getSimpleName() + "...");
            container.showHelp();
        } else {
            System.out.println("[" + getClass().getSimpleName() + "] End of chain — no help available.");
        }
    }

    public void setTooltipText(String text) {
        this.tooltipText = text;
    }

    public String getTooltipText() {
        return tooltipText;
    }
}
