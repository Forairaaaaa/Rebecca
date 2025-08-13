/// 走 C# 开个读取线程，将收到数据通过 signal 转发到主线程
using Godot;
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;

public partial class RebeccaImu : Node
{
    [Signal]
    public delegate void ImuDataReceivedEventHandler(string data);

    [Export]
    public string deviceId = "imu0";

    [Export]
    public string host = "localhost";

    [Export]
    public int port = 12580;

    private Thread _readerThread;
    private Process _process;
    private bool _running = false;
    private const string _cliFileName = "rebecca-imu";

    public override void _Ready()
    {
        GD.Print("start rebecca imu reading");
        SetDataPublishingEnable(true);
        StartReading();
    }

    /// <summary>
    /// Start or stop imu's data publishing
    /// </summary>
    /// <param name="enable"></param>
    /// <returns></returns>
    private bool SetDataPublishingEnable(bool enable)
    {
        var cmd = enable ? "start" : "stop";

        Process process = new();

        process.StartInfo.FileName = _cliFileName;
        process.StartInfo.Arguments = $"--host {host} --port {port} {deviceId} {cmd}";
        process.StartInfo.RedirectStandardOutput = true;
        process.StartInfo.RedirectStandardError = true;
        process.StartInfo.UseShellExecute = false;
        process.StartInfo.CreateNoWindow = true;

        process.Start();

        string output = process.StandardOutput.ReadToEnd();
        GD.Print($"{cmd} data publising get: ", output);

        process.WaitForExit();

        return process.ExitCode == 0;
    }

    /// <summary>
    /// Create a thread to consume every imu data output and send thme back to engine via signal
    /// </summary>
    public void StartReading()
    {
        if (_running)
        {
            GD.PrintErr("alreadly reading");
            return;
        }

        _process = new Process();
        _process.StartInfo.FileName = _cliFileName;
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

    private void ReadingWorker()
    {
        try
        {
            using StreamReader reader = _process.StandardOutput;
            string line;
            while (_running && (line = reader.ReadLine()) != null)
            {
                GD.Print(line);
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

    public void StopReading()
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

    public override void _ExitTree()
    {
        StopReading();
        SetDataPublishingEnable(false);
    }


    /// <summary>
    /// Stop all shit when window is about to close
    /// </summary>
    /// <param name="what"></param>
    public override void _Notification(int what)
    {
        if (what == NotificationWMCloseRequest)
        {
            GetTree().Quit();

            StopReading();
            SetDataPublishingEnable(false);
        }
    }
}
