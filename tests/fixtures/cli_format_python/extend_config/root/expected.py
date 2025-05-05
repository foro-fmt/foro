def extend_config_test():
        # This should use single quotes from parent config and 8-space indents from child config
        message = 'This string should use single quotes as defined in the parent config'
        if True:
                print(message)
                for i in range(5):
                        print(i)
        return 'Done'
