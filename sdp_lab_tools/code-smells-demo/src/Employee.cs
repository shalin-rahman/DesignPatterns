namespace CodeSmellsDemo;

// Large Class / God Class + Divergent Change: this one class changes for
// payroll-rule reasons, tax-law reasons, printing-format reasons,
// database-schema reasons, email-template reasons, and reporting reasons —
// six unrelated causes of change all landing in the same file.
public class Employee
{
    public string Name = "";
    public double BaseSalary;
    public double HoursWorked;
    public double OvertimeHours;

    // Changes when payroll rules change.
    public double CalculateSalary()
    {
        double overtimePay = OvertimeHours * (BaseSalary / 160) * 1.5;
        return BaseSalary + overtimePay;
    }

    // Changes when tax law changes.
    public double CalculateTax(double salary)
    {
        if (salary > 100000)
        {
            return salary * 0.25;
        }
        else if (salary > 50000)
        {
            return salary * 0.15;
        }
        else
        {
            return salary * 0.05;
        }
    }

    // Changes when the payslip layout changes.
    public void GeneratePaySlip(double salary, double tax)
    {
        Console.WriteLine("---- Pay Slip ----");
        Console.WriteLine($"Employee: {Name}");
        Console.WriteLine($"Gross: {salary}");
        Console.WriteLine($"Tax: {tax}");
        Console.WriteLine($"Net: {salary - tax}");
    }

    // Changes when the database schema changes.
    public void SaveToDatabase(double salary, double tax)
    {
        Console.WriteLine($"INSERT INTO payroll VALUES ('{Name}', {salary}, {tax})");
    }

    // Changes when the notification template changes.
    public void SendEmail(double salary)
    {
        Console.WriteLine($"Email to {Name}: your salary of {salary} has been processed.");
    }

    // Changes when reporting requirements change.
    public void GenerateReport()
    {
        Console.WriteLine($"Report: {Name} worked {HoursWorked} hours this period.");
    }

    // Changes when backup policy changes.
    public void BackupData()
    {
        Console.WriteLine($"Backing up payroll record for {Name}.");
    }
}
