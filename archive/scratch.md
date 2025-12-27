# The container is running with --capability=all, so this should work now!
sudo ./scripts/lab-env exec python /shared/pid_namespace.py

If you still get the "no system bus" error with exec, try entering the shell first:

# Enter the shell
sudo ./scripts/lab-env shell

# Once inside, run your experiment
python /shared/pid_namespace.py

Or use machinectl directly:

# Run command directly with machinectl
sudo machinectl shell root@miniarch /usr/bin/python /shared/pid_namespace.py