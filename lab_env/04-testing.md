Testing Checklist

  1. Status & Info Commands

  # Check container status (should show STOPPED since you exited)
  ./scripts/lab-env status

  # Get detailed info
  ./scripts/lab-env info

  2. Start/Stop Lifecycle

  # Start container manually
  sudo ./scripts/lab-env start

  # Check status again (should show RUNNING)
  ./scripts/lab-env status

  # Stop it
  sudo ./scripts/lab-env stop

  # Verify it stopped
  ./scripts/lab-env status

  3. Test Shared Directory

  # Create a test file on host
  echo "Hello from host" > /home/cip/.miniarch/test.txt

  # Enter container and check if file is visible
  ./scripts/lab-env shell
  # Inside container:
  # ls -la /shared
  # cat /shared/test.txt
  # exit

  4. Test Exec Command

  # Run command without entering shell
  ./scripts/lab-env exec pwd
  ./scripts/lab-env exec cat /shared/test.txt
  ./scripts/lab-env exec whoami

  # Test if raw IP works (network layer)
  ./scripts/lab-env exec ping -c 2 1.1.1.1

  # Test if DNS resolution works
  ./scripts/lab-env exec ping -c 2 archlinux.org

  # Check what DNS config the container has
  ./scripts/lab-env exec cat /etc/resolv.conf

  5. Test Package Management

  # Install a package
  sudo ./scripts/lab-env pkg install htop

  # Verify it's installed
  ./scripts/lab-env exec which htop
  ./scripts/lab-env exec htop --version

  6. Test Namespace Experiments (the whole point!)

  # Create a simple namespace test script in shared dir
  cat > /home/cip/.miniarch/ns_test.py << 'EOF'
  import os
  print(f"PID: {os.getpid()}")
  print(f"Hostname: {os.uname().nodename}")
  EOF

  # Run it in the container
  ./scripts/lab-env exec python /shared/ns_test.py

  7. Test Reset (destructive - do last)

  # This will destroy and recreate everything
  sudo ./scripts/lab-env reset

  Start with #1 (Status & Info) and work your way down. Let me know what you find or if anything breaks!