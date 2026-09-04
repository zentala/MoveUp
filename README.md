# MoveUp

MoveUp is an Electron-based desktop application designed to promote healthy work habits by encouraging users to take breaks and change their position regularly. It integrates seamlessly with Open Smart Desk to monitor the time you spend at your desk and uses gamification elements such as points and badges to keep you motivated.

## Features

- **Desk Time Monitoring**: Integrates with Open Smart Desk to track the time you spend sitting.
- **Break Reminders**: Timely notifications to remind you to take breaks and change positions.
- **Gamification**: Earn points and badges for taking breaks and staying active.
- **Future Integrations**: Planned support for smartbands and other fitness tracking devices.

## Getting Started

 ```sh
 git clone git@github.com:zentala/MoveUp.git
 cd moveup
 npm install
 npm run build:css
 npm run start
 ```

## Startup and PM3

The packaged desktop app registers itself to start with Windows and launches
hidden in the tray. When it is managed by PM3, PM3 is the single supervisor and
the desktop app does not register a second logon entry.

For this machine, from the repository directory:

```sh
pm3 reload
pm3 start moveup/dev
```

The supervised development interface is available at `http://moveup.internal`.
