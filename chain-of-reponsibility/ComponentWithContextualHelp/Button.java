/**
 * Concrete leaf handler representing a simple UI widget (request origin).
 */
public class Button extends Component {

    /** Creates a button without a tooltip (forwards requests to its container). */
    public Button() {
    }

    /** Creates a button with a tooltip (terminates the chain when requested). */
    public Button(String tooltipText) {
        setTooltipText(tooltipText);
    }
}
