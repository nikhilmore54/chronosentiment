import { API_BASE_URL } from '../config/api';

/** @param {string} path Must begin with `/` */
export const apiUrl = (path) => `${API_BASE_URL}${path}`;
