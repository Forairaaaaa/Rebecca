using Godot;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System;

[GlobalClass]
public partial class Imu : Node
{
    [Signal]
    public delegate void ImuDataReceivedEventHandler(string data);

    [Export]
    public string host = "localhost";

    [Export]
    public int port = 12580;

    [Export]
    public string deviceId = "imu0";

    [Export]
    public string cliToolName = "rebecca-imu";

    private Thread _readerThread;
    private Process _process;
    private bool _running = false;

    public override void _Ready()
    {
        // Check if tool is available
        if (!Common.ExecuteCommand(cliToolName, "--version"))
        {
            GD.PrintErr($"{cliToolName} is not available");
            return;
        }

        StartReading();
        StartWorker();
    }

    /// <summary>
    /// Start reading imu data
    /// </summary>
    /// <returns></returns>
    public bool StartReading()
    {
        var args = $"--host {host} --port {port} {deviceId} start";
        return Common.ExecuteCommand(cliToolName, args);
    }

    /// <summary>
    /// Stop reading imu data
    /// </summary>
    /// <returns></returns>
    public bool StopReading()
    {
        var args = $"--host {host} --port {port} {deviceId} stop";
        return Common.ExecuteCommand(cliToolName, args);
    }

    private void ReadingWorker()
    {
        try
        {
            using StreamReader reader = _process.StandardOutput;
            string line;
            while (_running && (line = reader.ReadLine()) != null)
            {
                // GD.Print(line);
                CallDeferred(nameof(EmitReceivedData), line);
            }
        }
        catch (Exception e)
        {
            GD.PrintErr("Reader process error: ", e);
        }
    }

    private void EmitReceivedData(string data)
    {
        EmitSignal(SignalName.ImuDataReceived, data);
    }

    private void StartWorker()
    {
        if (_running)
        {
            GD.PrintErr("imu reading worker already running");
            return;
        }

        _process = new Process();
        _process.StartInfo.FileName = cliToolName;
        _process.StartInfo.Arguments = $"--host {host} --port {port} {deviceId} read";
        _process.StartInfo.UseShellExecute = false;
        _process.StartInfo.RedirectStandardOutput = true;
        _process.StartInfo.RedirectStandardError = true;
        _process.StartInfo.CreateNoWindow = true;

        _process.Start();
        _running = true;

        _readerThread = new Thread(ReadingWorker)
        {
            IsBackground = true
        };
        _readerThread.Start();
    }

    private void StopWroker()
    {
        _running = false;

        try
        {
            if (!_process.HasExited)
                _process.Kill();
        }
        catch (Exception e)
        {
            GD.PrintErr("Kill reading process failed: ", e);
        }

        _readerThread?.Join();
    }
}
