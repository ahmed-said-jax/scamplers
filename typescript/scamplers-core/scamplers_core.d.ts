/* tslint:disable */
/* eslint-disable */
export enum InstitutionOrdinalColumn {
  Name = 0,
}
export enum PersonOrdinalColumn {
  Name = 0,
  Email = 1,
}
export enum UserRole {
  AppAdmin = 0,
  ComputationalStaff = 1,
  BiologyStaff = 2,
  Unknown = 3,
}
export class Client {
  free(): void;
  send_new_institution(data: NewInstitution, api_key?: string | null): Promise<Institution>;
  send_new_person(data: NewPerson, api_key?: string | null): Promise<Person>;
  send_new_ms_login(data: NewPerson, api_key?: string | null): Promise<CreatedUser>;
  constructor(backend_url: string, token: string);
}
export class CreatedUser {
  private constructor();
  free(): void;
  person: Person;
  get api_key(): string;
  set api_key(value: string | null | undefined);
}
export class Institution {
  private constructor();
  free(): void;
  id: string;
  name: string;
  link: string;
}
export class InstitutionOrdering {
  private constructor();
  free(): void;
  static new(): InstitutionOrderingBuilder;
  column: InstitutionOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`InstitutionOrdering`](struct.InstitutionOrdering.html).
 */
export class InstitutionOrderingBuilder {
  private constructor();
  free(): void;
  column(value: InstitutionOrdinalColumn): InstitutionOrderingBuilder;
  descending(value: boolean): InstitutionOrderingBuilder;
  /**
   * Builds a new `InstitutionOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): InstitutionOrdering;
}
export class InstitutionOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class InstitutionQuery {
  private constructor();
  free(): void;
  static new(): InstitutionQueryBuilder;
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  order_by: InstitutionOrdering[];
  pagination: Pagination;
}
/**
 * Builder for [`InstitutionQuery`](struct.InstitutionQuery.html).
 */
export class InstitutionQueryBuilder {
  private constructor();
  free(): void;
  ids(value: string[]): InstitutionQueryBuilder;
  name(value?: string | null): InstitutionQueryBuilder;
  order_by(value: InstitutionOrdering[]): InstitutionQueryBuilder;
  pagination(value: Pagination): InstitutionQueryBuilder;
  /**
   * Builds a new `InstitutionQuery`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): InstitutionQuery;
}
export class InstitutionQueryError {
  private constructor();
  free(): void;
  error(): string;
}
export class InstitutionReference {
  private constructor();
  free(): void;
  id: string;
  link: string;
}
export class InstitutionSummary {
  private constructor();
  free(): void;
  id: string;
  name: string;
  link: string;
}
export class NewInstitution {
  private constructor();
  free(): void;
  static new(): NewInstitutionBuilder;
  id: string;
  name: string;
}
/**
 * Builder for [`NewInstitution`](struct.NewInstitution.html).
 */
export class NewInstitutionBuilder {
  private constructor();
  free(): void;
  id(value: string): NewInstitutionBuilder;
  name(value: string): NewInstitutionBuilder;
  /**
   * Builds a new `NewInstitution`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewInstitution;
}
export class NewInstitutionError {
  private constructor();
  free(): void;
  error(): string;
}
export class NewPerson {
  private constructor();
  free(): void;
  static new(): NewPersonBuilder;
  name: string;
  email: string;
  get orcid(): string;
  set orcid(value: string | null | undefined);
  institution_id: string;
  get ms_user_id(): string;
  set ms_user_id(value: string | null | undefined);
  roles: any[];
}
/**
 * Builder for [`NewPerson`](struct.NewPerson.html).
 */
export class NewPersonBuilder {
  private constructor();
  free(): void;
  name(value: string): NewPersonBuilder;
  email(value: string): NewPersonBuilder;
  orcid(value?: string | null): NewPersonBuilder;
  institution_id(value: string): NewPersonBuilder;
  ms_user_id(value?: string | null): NewPersonBuilder;
  roles(value: any[]): NewPersonBuilder;
  /**
   * Builds a new `NewPerson`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): NewPerson;
}
export class NewPersonError {
  private constructor();
  free(): void;
  error(): string;
}
export class Pagination {
  private constructor();
  free(): void;
  static new(): PaginationBuilder;
  limit: bigint;
  offset: bigint;
}
/**
 * Builder for [`Pagination`](struct.Pagination.html).
 */
export class PaginationBuilder {
  private constructor();
  free(): void;
  limit(value: bigint): PaginationBuilder;
  offset(value: bigint): PaginationBuilder;
  /**
   * Builds a new `Pagination`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): Pagination;
}
export class PaginationError {
  private constructor();
  free(): void;
  error(): string;
}
export class Person {
  private constructor();
  free(): void;
  id: string;
  name: string;
  link: string;
  email: string;
  get orcid(): string;
  set orcid(value: string | null | undefined);
  institution: Institution;
}
export class PersonOrdering {
  private constructor();
  free(): void;
  static new(): PersonOrderingBuilder;
  column: PersonOrdinalColumn;
  descending: boolean;
}
/**
 * Builder for [`PersonOrdering`](struct.PersonOrdering.html).
 */
export class PersonOrderingBuilder {
  private constructor();
  free(): void;
  column(value: PersonOrdinalColumn): PersonOrderingBuilder;
  descending(value: boolean): PersonOrderingBuilder;
  /**
   * Builds a new `PersonOrdering`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonOrdering;
}
export class PersonOrderingError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonQuery {
  private constructor();
  free(): void;
  static new(): PersonQueryBuilder;
  ids: string[];
  get name(): string;
  set name(value: string | null | undefined);
  get email(): string;
  set email(value: string | null | undefined);
  order_by: PersonOrdering[];
  pagination: Pagination;
}
/**
 * Builder for [`PersonQuery`](struct.PersonQuery.html).
 */
export class PersonQueryBuilder {
  private constructor();
  free(): void;
  ids(value: string[]): PersonQueryBuilder;
  name(value?: string | null): PersonQueryBuilder;
  email(value?: string | null): PersonQueryBuilder;
  order_by(value: PersonOrdering[]): PersonQueryBuilder;
  pagination(value: Pagination): PersonQueryBuilder;
  /**
   * Builds a new `PersonQuery`.
   *
   * # Errors
   *
   * If a required field has not been initialized.
   */
  build(): PersonQuery;
}
export class PersonQueryError {
  private constructor();
  free(): void;
  error(): string;
}
export class PersonReference {
  private constructor();
  free(): void;
  id: string;
  link: string;
}
export class PersonSummary {
  private constructor();
  free(): void;
  id: string;
  name: string;
  link: string;
  email: string;
  get orcid(): string;
  set orcid(value: string | null | undefined);
}
