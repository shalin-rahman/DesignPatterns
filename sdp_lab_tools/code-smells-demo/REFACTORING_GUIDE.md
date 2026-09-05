# Refactoring guide

This project is one small, working "campus store + payroll" app that
deliberately contains all 12 code smells from the lecture, wired together
the way a real messy codebase would have them — not 12 separate toy
snippets. Run it with `dotnet run` from this folder; nothing here is
broken, it just isn't clean.

Each section below says where a smell lives, why it's a smell, and what
to do about it. "Design pattern" is only listed where one genuinely fits —
per the lecture, a smell does not automatically need a pattern.

## 1. Long Method

**Where:** `OrderService.ProcessOrder` (`src/OrderService.cs`).

It validates the order, computes the subtotal, applies the discount,
picks a payment description, charges the payment, writes the invoice
line, sends the notification, saves the record, and prints a snapshot —
eight jobs in one method.

**Why it's a smell:** you can't test "how is total computed" without also
running payment and notification. Reading the method means holding all
eight steps in your head at once.

**Fix:** Extract Method for each step (`ValidateItems`, `CalculateSubtotal`,
`DescribePaymentType`, `RecordPayment`), so `ProcessOrder` becomes a short
list of calls.

## 2. Large Class / God Class

**Where:** `Employee` (`src/Employee.cs`).

One class calculates salary, calculates tax, formats a payslip, writes to
a database, sends email, builds a report, and runs backups.

**Why it's a smell:** the class has no single reason to exist — it's
"employee stuff," a grab-bag.

**Fix:** Extract Class into `SalaryCalculator`, `TaxCalculator`,
`PaySlipPrinter`, `PayrollRepository`, `PayrollNotifier`. `Employee`
becomes plain data; each new class owns one responsibility.

## 3. Duplicated Code

**Where:** `Student.CalculateResult` and `Instructor.CalculateClassAverage`
(`src/Student.cs`, `src/Instructor.cs`).

Both sum three scores, divide by three, and map the average to the same
grade bands — identical logic, different field names.

**Why it's a smell:** a bug in the grading bands has to be fixed twice,
and it's easy to fix one copy and forget the other.

**Fix:** Extract the shared logic into one `GradeCalculator.FromAverage(double)`
method (or a shared base type) that both classes call.

## 4. Long Parameter List

**Where:** `StudentService.CreateStudent` (`src/StudentService.cs`).

Eight positional string parameters (name, id, department, email, phone,
address, semester, cgpa) that the caller must supply in exact order.

**Why it's a smell:** callers can silently swap two same-typed arguments
(e.g. `email` and `phone`) and the compiler won't catch it.

**Fix:** Introduce Parameter Object — a `StudentRegistration` record
holding the same fields, passed as one argument. **Builder** is a good
fit here too, since the lecture's chain calls this out explicitly:
`StudentRegistrationBuilder` lets a caller set only the fields it has.

## 5. Feature Envy

**Where:** `Order.CalculateDiscount` (`src/Order.cs`).

Every branch reads `Customer.Age`, `Customer.MembershipYears`,
`Customer.PurchaseHistoryCount`, `Customer.Account.Balance` — the method
is about `Customer`, not about `Order`.

**Why it's a smell:** the discount rule lives in the wrong class, so
`Customer` can never be reused with different discount rules, and every
Order-side change risks touching Customer's data shape.

**Fix:** Move Method — put `CalculateDiscount()` on `Customer` itself
(it already has everything it needs), and have `Order` call
`Customer.CalculateDiscount()`.

## 6. Data Clumps

**Where:** `Order.ShipTo` and `Order.BillTo` (`src/Order.cs`).

`street`, `city`, `postalCode`, `country` always travel together as four
separate parameters — and `Address` (`src/Address.cs`) already models
exactly this group.

**Why it's a smell:** the four values are only meaningful together; passing
them separately means every call site can (and must) get the order right
without any type enforcing it.

**Fix:** Introduce Parameter Object — change both methods to take a single
`Address` parameter instead of four primitives.

## 7. Switch Statements

**Where:** `PaymentService.Pay`, `PaymentController.HandlePaymentRequest`,
`InvoiceService.DescribePaymentMethod`, `NotificationService.NotifyPaymentReceived`,
`DatabaseService.SavePaymentRecord`, and the switch inside
`OrderService.ProcessOrder` — six copies of the same
`CASH` / `CARD` / `BKASH` / `NAGAD` branching.

**Why it's a smell:** this is the same smell as Shotgun Surgery below,
seen from the "how is each individual method written" angle — a `switch`
on a type code is usually a sign the branches should be types, not strings.

**Fix / pattern:** Replace Conditional with Polymorphism — introduce an
`IPaymentMethod` with `Charge`, `Describe`, `NotifyReceived`, `SaveRecord`,
and one implementation per type (`CashPayment`, `CardPayment`,
`BkashPayment`, `NagadPayment`). This is the **Strategy** pattern from the
lecture's chain: each concrete `IPaymentMethod` is a strategy the caller
picks at runtime instead of a string it branches on.

## 8. Primitive Obsession

**Where:** `Student` (`src/Student.cs`) — `Email`, `Phone`, `Cgpa`, `Semester`
are all bare `string`s.

**Why it's a smell:** nothing stops an invalid email or an out-of-range
CGPA from being stored, and validation logic (like the `Contains("@")`
check in `StudentService`) has to be re-added at every call site instead
of living in one place.

**Fix:** Replace Data Value with Object — small value types
(`EmailAddress`, `PhoneNumber`, `Cgpa`) that validate themselves in their
constructor, so an invalid value can't exist at all.

## 9. Divergent Change

**Where:** `Employee` again (`src/Employee.cs`), viewed from the "what
makes it change" angle rather than the "how big is it" angle.

`CalculateSalary`/`CalculateTax` change when payroll or tax rules change;
`GeneratePaySlip` changes when the payslip layout changes; `SaveToDatabase`
changes when the schema changes; `SendEmail` changes when the notification
template changes; `GenerateReport`/`BackupData` change for reporting and
backup-policy reasons. Six unrelated reasons to edit one file.

**Why it's a smell:** an unrelated policy change (say, a new tax bracket)
forces you to open and recompile a class that also handles email and
backups, with no isolation between those concerns.

**Fix:** the same Extract Class split as item 2 also fixes this — once
each responsibility has its own class, each one changes for exactly one
reason.

## 10. Shotgun Surgery

**Where:** the same six-file cluster as item 7
(`PaymentService`, `PaymentController`, `InvoiceService`,
`NotificationService`, `DatabaseService`, `OrderService`).

**Why it's a smell:** adding a fifth payment type (say `ROCKET`) means
finding and editing all six switches correctly, in the same way, or the
new type breaks somewhere no one thought to check.

**Fix / pattern:** same as item 7 — once payment types are polymorphic
`IPaymentMethod` implementations, adding a new type means adding one new
class, not editing six existing ones. A **Factory Method** (or simple
factory) that maps `"CARD"` → `new CardPayment()` is the natural place to
create the right strategy from external input (e.g. a request string),
matching the lecture's Problem → Smell → Refactoring → Pattern chain.

## 11. Inappropriate Intimacy

**Where:** `Order.PrintCustomerSnapshot` (`src/Order.cs`), reaching three
levels deep: `Customer.Address.City`, `Customer.Account.Balance`,
`Customer.Profile.Membership.Type`.

**Why it's a smell:** `Order` now depends on the exact internal shape of
`Customer`, `Address`, `Account`, `Profile`, and `Membership` all at once —
any of those five classes changing its structure can break `Order`.

**Fix:** Hide Delegate — give `Customer` its own
`DescribeSnapshot()` method that reaches into its own `Address`/`Account`/
`Profile` (which it's allowed to know about), and have `Order` call that
one method instead of reaching through it.

## 12. Speculative Generality

**Where:** `src/PaymentGateway.cs` —
`AbstractPaymentGatewayFactoryProvider` → `IPaymentGatewayFactory` →
`IPaymentGateway`, three abstraction layers built for a second gateway
provider that never arrives. `CardPaymentGateway` is the only
implementation, and `Program.cs` only ever asks for exactly that one
concrete type.

**Why it's a smell:** every layer here is unused flexibility — it makes
the code harder to navigate (three files and three types to add one
`Charge` call) without buying anything, since nothing in the app varies
along the axis this hierarchy was built to support.

**Fix:** Collapse Hierarchy — delete
`AbstractPaymentGatewayFactoryProvider`, `IPaymentGatewayFactory`, and
`CardPaymentGatewayFactory`; keep a single concrete `CardPaymentGateway`
class (or fold the abstraction back in only if and when a second real
gateway is actually needed — YAGNI: build the abstraction from the second
real case, not in anticipation of it).
