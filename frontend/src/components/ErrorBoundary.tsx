import { Component, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('ErrorBoundary:', error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          className="flex flex-col items-center justify-center py-16 px-4 text-center"
          style={{ minHeight: '60vh' }}
        >
          <span className="text-5xl mb-4">😵</span>
          <p
            className="text-base mb-4"
            style={{ color: 'var(--tg-text-color)' }}
          >
            Что-то пошло не так
          </p>
          <button
            onClick={() => this.setState({ hasError: false })}
            className="px-6 py-2.5 rounded-xl text-sm font-medium"
            style={{
              backgroundColor: 'var(--tg-button-color)',
              color: 'var(--tg-button-text-color)',
            }}
          >
            Попробовать снова
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
