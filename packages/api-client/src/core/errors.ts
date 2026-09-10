import type { ApiErrorData, LumenErrorResponse } from '../types/errors'
import { isLumenErrorResponse } from '../types/errors'

/**
 * Base error class for all Lumen API errors
 */
export class LumenApiError extends Error {
	/**
	 * HTTP status code (if available)
	 */
	readonly statusCode?: number

	/**
	 * Original error that was caught
	 */
	readonly originalError?: Error

	/**
	 * Response data from the API (if available)
	 */
	readonly responseData?: unknown

	/**
	 * Error context (e.g., module name, operation being performed)
	 */
	readonly context?: string

	constructor(message: string, data?: ApiErrorData) {
		super(message)
		this.name = 'LumenApiError'

		this.statusCode = data?.statusCode
		this.originalError = data?.originalError
		this.responseData = data?.responseData
		this.context = data?.context

		// Maintains proper stack trace for where our error was thrown (only available on V8)
		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, LumenApiError)
		}
	}

	/**
	 * Create a LumenApiError from an unknown error
	 */
	static fromUnknown(error: unknown, context?: string): LumenApiError {
		if (error instanceof LumenApiError) {
			return error
		}

		if (error instanceof Error) {
			return new LumenApiError(error.message, {
				originalError: error,
				context,
			})
		}

		return new LumenApiError(String(error), { context })
	}
}

/**
 * Error class for Lumen server errors (kyros/archon)
 * Extends LumenApiError with V1 error response parsing
 */
export class LumenServerError extends LumenApiError {
	/**
	 * V1 error information (if available)
	 */
	readonly v1Error?: LumenErrorResponse

	constructor(message: string, data?: ApiErrorData & { v1Error?: LumenErrorResponse }) {
		// If we have a V1 error, format the message nicely
		let errorMessage = message
		if (data?.v1Error) {
			errorMessage = `[${data.v1Error.error}] ${data.v1Error.description}`
			if (data.v1Error.context) {
				errorMessage = `${data.v1Error.context}: ${errorMessage}`
			}
		}

		super(errorMessage, data)
		this.name = 'LumenServerError'
		this.v1Error = data?.v1Error

		if (Error.captureStackTrace) {
			Error.captureStackTrace(this, LumenServerError)
		}
	}

	/**
	 * Create a LumenServerError from response data
	 */
	static fromResponse(
		statusCode: number,
		responseData: unknown,
		context?: string,
	): LumenServerError {
		const v1Error = isLumenErrorResponse(responseData) ? responseData : undefined

		let message = `HTTP ${statusCode}`
		if (v1Error) {
			message = v1Error.description
		} else if (typeof responseData === 'string') {
			message = responseData
		}

		return new LumenServerError(message, {
			statusCode,
			responseData,
			context,
			v1Error,
		})
	}

	/**
	 * Create a LumenServerError from an unknown error
	 */
	static fromUnknown(error: unknown, context?: string): LumenServerError {
		if (error instanceof LumenServerError) {
			return error
		}

		if (error instanceof LumenApiError) {
			return new LumenServerError(error.message, {
				statusCode: error.statusCode,
				originalError: error.originalError,
				responseData: error.responseData,
				context: context ?? error.context,
			})
		}

		if (error instanceof Error) {
			return new LumenServerError(error.message, {
				originalError: error,
				context,
			})
		}

		return new LumenServerError(String(error), { context })
	}
}
