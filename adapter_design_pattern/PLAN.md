# Implementation Plan: Adapter Design Pattern (Online Store Payment System)

## Objective
Implement an online store payment system following the Adapter Design Pattern as specified in `Assignment Adapter Design Pattern.pdf`.

## Architecture & Components
- **Target Interface**: `PaymentProcessor` (`getName()`, `pay(String customer, double amountInDollars)`, `refund(String customer, double amountInDollars)`)
- **Adaptees (3rd-party Services)**:
  - `LegacyBankGateway`: `int makeTransfer(String, long)`, `int reverseTransfer(String, long)` (cents, status code 0 = success, max 100000 cents limit).
  - `QuickPaySDK`: `String sendMoney(String currency, double amount, String payerName)`, `boolean returnMoney(String transactionId)` (returns txId, refund needs txId).
  - `EuroPayService`: `boolean chargeCard(String email, double euros)`, `boolean refundCard(String email, double euros)` (1 USD = 0.90 EUR, customer email = `<name>@store.com`).
- **Adapters**:
  - `LegacyBankAdapter`
  - `QuickPayAdapter` (maintains customer-to-transactionId mapping)
  - `EuroPayAdapter`
- **Client**: `Checkout` (interacts solely with `PaymentProcessor`, no if/else type checks, logs transaction output).
- **Driver**: `Main` (runs the 6 sample transactions and demonstrates extensibility).

## Execution Steps
1. **Source Code Implementation**:
   - `PaymentProcessor.java`
   - `LegacyBankGateway.java`, `QuickPaySDK.java`, `EuroPayService.java`
   - `LegacyBankAdapter.java`, `QuickPayAdapter.java`, `EuroPayAdapter.java`
   - `Checkout.java`
   - `Main.java`
2. **Compilation & Verification**:
   - Compile all `.java` files using `javac`.
   - Run `Main` and verify output matches expected sample data exactly.
3. **UML Diagrams**:
   - Generate standard UML Class Diagram SVG (`uml-class-diagram.svg`).
   - Generate UML Sequence Diagram SVG for `Checkout.refund("Bob", 100.00)` (`uml-sequence-diagram.svg`).
4. **Documentation**:
   - Create comprehensive `README.md` fulfilling Task 1 (answers & table), Task 2 (diagrams & explanations), and Task 3 (instructions, code walkthrough, 4th service extension guide).

