using System.IO;
using System.Threading;

public class Imu
{
    public string Host { get; set; } = "localhost";
    public int Port { get; set; } = 12580;
    public string DeviceId { get; set; } = "imu0";
    public string CliToolName { get; set; } = "rebecca-imu";

    public bool Init()
    {
        // Check if tool is available
        if (!Common.ExecuteCommand(CliToolName, "--version"))
        {
            return false;
        }

        return true;
    }
}
