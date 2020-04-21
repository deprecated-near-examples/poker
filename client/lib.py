import re
import json
import relaxedjson
from subprocess import PIPE, Popen, check_output
import logging

logging.basicConfig(filename='poker.log', filemode='a+',
                    format='%(asctime)s - %(levelname)s - %(message)s', level=logging.DEBUG)

ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')


def get_account_id(path):
    with open(path) as f:
        data = relaxedjson.parse(f.read())
    return data['account_id']


def parse(inp):
    inp = ansi_escape.sub('', inp)
    logging.debug(f"Parsing: {repr(inp)}")
    return relaxedjson.parse(inp)


class Command:
    def __init__(self, short, name, help, callback):
        self.short = short
        self.name = name
        self.help = help
        self.callback = callback


class Near:
    def __init__(self, node_key_path, contract, node_url):
        self.node_key_path = node_key_path
        self.contract = contract
        self.node_url = node_url
        self.account_id = get_account_id(node_key_path)

    def _parse(self, output):
        lines = output.strip('\n').split('\n')
        pos = 0
        while pos < len(lines) and not lines[pos].startswith("Loaded"):
            pos += 1

        if pos == len(lines):
            raise ValueError(f"Error parsing output: {output}")

        output = '\n'.join(lines[pos + 1:])
        return parse(output)

    def add_command_url(self, command):
        if self.node_url:
            command.extend(["--nodeUrl", self.node_url])
        return command

    def view(self, name, args={}):
        command = [
            "near",
            "view",
            self.contract,
            name,
            json.dumps(args),
            "--keyPath",
            self.node_key_path,
            "--masterAccount",
            self.account_id
        ]
        command = self.add_command_url(command)

        logging.debug(f"View Command: {command}")
        proc = Popen(command, stdout=PIPE, stderr=PIPE)

        ret = proc.wait()
        logging.debug(f"Exit code: {ret}")
        if ret == 0:
            result = proc.stdout.read().decode()
            return self._parse(result)
        else:
            logging.warn(f"Command stdout: {proc.stdout.read().decode()}")

    def change(self, name, args={}):
        command = [
            "near",
            "call",
            self.contract,
            name,
            json.dumps(args),
            "--keyPath",
            self.node_key_path,
            "--accountId",
            self.account_id
        ]
        command = self.add_command_url(command)

        logging.debug(f"Change Command: {command}")
        proc = Popen(command, stdout=PIPE, stderr=PIPE)

        ret = proc.wait()
        logging.debug(f"Exit code: {ret}")
        if ret == 0:
            result = proc.stdout.read().decode()
            return self._parse(result)
        else:
            logging.warn(f"Command stdout: {proc.stdout.read().decode()}")


def register(function=None, *, short=None, name=None, help=""):
    if function is None:
        def dec(function):
            function._command = True
            function._name = name or function.__name__
            function._short = short or function._name[0]
            function._help = help
            return function
        return dec
    else:
        function._command = True
        function._name = name or function.__name__
        function._short = short or function._name[0]
        function._help = help
        return function


class App:
    def __init__(self, node_key_path, contract, node_url, ui):
        self.near = Near(node_key_path, contract, node_url)
        self.ui = ui
        self.ui.set_account_id(self.near.account_id)
        self._commands = {}
        for func_name in dir(self):
            if func_name.startswith('__'):
                continue
            func = self.__getattribute__(func_name)
            if '_command' in dir(func):
                self._register(func._short, func._name, func._help, func)

    @property
    def account_id(self):
        return self.near.account_id

    def get_account_id(self):
        print(self.account_id)

    def _register(self, short, name, help, callback):
        assert not short in self._commands
        assert not name in self._commands
        command = Command(short, name, help, callback)
        self._commands[name] = command
        self._commands[short] = command

    @register(help="Show this help")
    def help(self, *args):
        print("Available commands:")
        for key, command in sorted(self._commands.items()):
            if key == command.name:
                parts = command.help.split('|')
                if len(parts) == 2:
                    _args = parts[0]
                    _expl = parts[1].strip()
                elif len(parts) == 1:
                    _args = ''
                    _expl = parts[0].strip()
                else:
                    _args = ''
                    _expl = ''
                print(f"[{command.short}]{command.name:<12} {_expl}")
                print((" " * 16) + f"args: {_args}")
                print()

    def feed(self, command):
        command = command.strip(' ')
        if not command:
            return

        comm, *args = command.split()

        if not comm in self._commands:
            print(f"Command [{comm}] not found.")
            self.help()
        else:
            callback = self._commands[comm].callback
            try:
                callback(*args)
            except Exception as e:
                print(*e.args)
        print()

    def start(self):
        print(
            f"Welcome to poker game {self.account_id}. Print [h]help for options.")
        logging.info(f"Start game with: {self.account_id}")

        while True:
            self.ui.display()
            command = input()
            self.feed(command)
