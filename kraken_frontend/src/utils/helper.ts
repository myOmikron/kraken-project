/**
 * Sleeps x milliseconds async
 *
 * @param timeout Sleep time in milliseconds
 */
export async function sleep(timeout: number): Promise<null> {
    return await new Promise((resolve) => setTimeout(resolve, timeout));
}
