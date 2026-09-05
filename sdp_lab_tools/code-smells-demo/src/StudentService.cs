namespace CodeSmellsDemo;

public class StudentService
{
    // Long Parameter List: eight positional strings the caller must get
    // in the right order, with no grouping by what they represent.
    public Student CreateStudent(
        string name,
        string id,
        string department,
        string email,
        string phone,
        string address,
        string semester,
        string cgpa)
    {
        if (!email.Contains("@"))
        {
            throw new ArgumentException("Invalid email.");
        }

        return new Student
        {
            Name = name,
            Id = id,
            Department = department,
            Email = email,
            Phone = phone,
            Address = address,
            Semester = semester,
            Cgpa = cgpa,
        };
    }
}
