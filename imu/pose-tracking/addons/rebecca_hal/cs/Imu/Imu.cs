using Godot;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System;

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
    public string cliToolName = "rebecca-hal";

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

        GD.Print("[IMU] ready");
    }

    /// <summary>
    /// Start reading imu data
    /// </summary>
    /// <returns></returns>
    public bool StartReading()
    {
        GD.Print("[IMU] start reading");
        var args = $"--host {host} --port {port} imu {deviceId} start";
        return Common.ExecuteCommand(cliToolName, args);
    }

    /// <summary>
    /// Stop reading imu data
    /// </summary>
    /// <returns></returns>
    public bool StopReading()
    {
        GD.Print("[IMU] stop reading");
        var args = $"--host {host} --port {port} imu {deviceId} stop";
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
        _process.StartInfo.Arguments = $"--host {host} --port {port} imu {deviceId} read";
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

    public void Cleanup()
    {
        GD.Print("[IMU] cleanup");
        StopWroker();
        StopReading();
    }

    public override void _ExitTree()
    {
        Cleanup();
    }

    public override void _Notification(int what)
    {
        if (what == NotificationWMCloseRequest)
        {
            GetTree().Quit();

            Cleanup();
        }
    }
}