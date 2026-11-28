/**
 * Concrete intermediate container handler that can display a modal help popup.
 */
public class Panel extends Container {

    private final String modalHelpText;

    public Panel() {
        this.modalHelpText = null;
    }

    public Panel(String modalHelpText) {
        this.modalHelpText = modalHelpText;
    }

    /**
     * Shows modal help popup if available; otherwise forwards to parent container.
     */
    @Override
    public void showHelp() {
        if (modalHelpText != null) {
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

    public String getModalHelpText() {
        return modalHelpText;
    }
}
