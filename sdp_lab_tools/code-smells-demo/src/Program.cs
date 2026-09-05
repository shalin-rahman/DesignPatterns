namespace CodeSmellsDemo;

public class Program
{
    public static void Main()
    {
        Console.WriteLine("=== Student enrollment ===");
        var studentService = new StudentService();
        var student = studentService.CreateStudent(
            "Rafi Karim",
            "S12345",
            "CSE",
            "rafi@example.com",
            "01700000000",
            "12 College Road, Dhaka",
            "Fall 2026",
            "3.60");
        student.Mark1 = 78;
        student.Mark2 = 85;
        student.Mark3 = 90;
        Console.WriteLine($"{student.Name} result: {student.CalculateResult()}");

        Console.WriteLine();
        Console.WriteLine("=== Instructor class average ===");
        var instructor = new Instructor { Name = "Dr. Nasrin Akter" };
        instructor.SampleScore1 = 88;
        instructor.SampleScore2 = 91;
        instructor.SampleScore3 = 76;
        Console.WriteLine($"Class average grade: {instructor.CalculateClassAverage()}");

        Console.WriteLine();
        Console.WriteLine("=== Payroll run ===");
        var employee = new Employee
        {
            Name = "Farhan Chowdhury",
            BaseSalary = 60000,
            HoursWorked = 176,
            OvertimeHours = 12,
        };
        double salary = employee.CalculateSalary();
        double tax = employee.CalculateTax(salary);
        employee.GeneratePaySlip(salary, tax);
        employee.SaveToDatabase(salary, tax);
        employee.SendEmail(salary);
        employee.GenerateReport();
        employee.BackupData();

        Console.WriteLine();
        Console.WriteLine("=== Order processing ===");
        var order = new Order
        {
            Customer = new Customer
            {
                Name = "Mahin Sarker",
                Age = 65,
                MembershipYears = 6,
                PurchaseHistoryCount = 60,
                Address = new Address
                {
                    Street = "45 Lake View",
                    City = "Chattogram",
                    PostalCode = "4000",
                    Country = "Bangladesh",
                },
                Account = new Account { Balance = 1500 },
                Profile = new Profile { Membership = new Membership { Type = "Gold" } },
            },
        };
        order.Items.Add(new Item { Price = 500, Quantity = 2 });
        order.Items.Add(new Item { Price = 250, Quantity = 3 });

        order.ShipTo("45 Lake View", "Chattogram", "4000", "Bangladesh");
        order.BillTo("45 Lake View", "Chattogram", "4000", "Bangladesh");

        var orderService = new OrderService();
        orderService.ProcessOrder(order, "CARD");

        var paymentController = new PaymentController();
        paymentController.HandlePaymentRequest("BKASH", 200);
        paymentController.HandlePaymentRequest("NAGAD", 150);
        paymentController.HandlePaymentRequest("CASH", 50);

        Console.WriteLine();
        Console.WriteLine("=== Speculative payment gateway hierarchy ===");
        AbstractPaymentGatewayFactoryProvider provider = new CardPaymentGatewayFactoryProvider();
        IPaymentGatewayFactory factory = provider.GetFactory();
        IPaymentGateway gateway = factory.Create();
        gateway.Charge(200);
    }
}
