using System.Diagnostics;

public static class Common
{
    /// <summary>
    /// 执行命令行工具并检查是否成功执行（返回码为0）
    /// </summary>
    /// <param name="fileName">可执行文件名</param>
    /// <param name="arguments">命令行参数</param>
    /// <returns>如果命令执行成功（返回码为0）则返回true，否则返回false</returns>
    public static bool ExecuteCommand(string fileName, string arguments = "")
    {
        try
        {
            var processStartInfo = new ProcessStartInfo
            {
                FileName = fileName,
                Arguments = arguments,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var process = Process.Start(processStartInfo);
            if (process == null)
            {
                return false;
            }

            process.WaitForExit();
            return process.ExitCode == 0;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// 执行命令行工具并获取输出结果
    /// </summary>
    /// <param name="fileName">可执行文件名</param>
    /// <param name="arguments">命令行参数</param>
    /// <returns>包含是否成功、标准输出和错误输出的结果</returns>
    public static (bool success, string output, string error) ExecuteCommandWithOutput(string fileName, string arguments = "")
    {
        try
        {
            var processStartInfo = new ProcessStartInfo
            {
                FileName = fileName,
                Arguments = arguments,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var process = Process.Start(processStartInfo);
            if (process == null)
            {
                return (false, "", "Process failed to start");
            }

            var output = process.StandardOutput.ReadToEnd();
            var error = process.StandardError.ReadToEnd();
            process.WaitForExit();

            return (process.ExitCode == 0, output, error);
        }
        catch (System.Exception ex)
        {
            return (false, "", ex.Message);
        }
    }
}